mod fluff;

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::{terminal, ExecutableCommand};
use std::{
    error::Error,
    io::{self, Stdout, Write},
    process::{Command, Stdio},
    sync::Arc,
    sync::Mutex,
};
use tui::{backend::CrosstermBackend, Terminal};
use wasmer::{Exports, Function, Instance, Module, Store, Value, JIT};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_wasi::WasiState;

static ROOT_PATH: &str = ".";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = Store::new(&JIT::new(&Singlepass::default()).engine());

    println!("Compiling module...");
    // FIXME: Switch to a higher performance compiler (`Store::default()`) and cache this on disk
    // I could use `(de)serialize_to_file()` for that
    let module = Module::from_file(&store, "target/wasm32-wasi/debug/module.wasm")?;

    // FIXME: Upstream the `Pipe` struct
    let output = fluff::Pipe::new();
    let input = fluff::Pipe::new();
    let mut wasi_env = WasiState::new("mosaic")
        .env("CLICOLOR_FORCE", "1")
        .preopen(|p| {
            p.directory(ROOT_PATH)
                .alias(".")
                .read(true)
                .write(true)
                .create(true)
        })?
        .stdin(Box::new(input))
        .stdout(Box::new(output))
        .finalize()?;

    let mut import_object = wasi_env.import_object(&module)?;
    // FIXME: Upstream an `ImportObject` merge method
    let mut host_exports = Exports::new();
    host_exports.insert(
        "host_open_file",
        Function::new_native_with_env(&store, Arc::clone(&wasi_env.state), host_open_file),
    );
    import_object.register("mosaic", host_exports);
    let instance = Instance::new(&module, &import_object)?;

    // WASI requires to explicitly set the memory for the `WasiEnv`
    wasi_env.set_memory(instance.exports.get_memory("memory")?.clone());

    let start = instance.exports.get_function("_start")?;
    let handle_key = instance.exports.get_function("handle_key")?;
    let draw = instance.exports.get_function("draw")?;

    // This eventually calls the `.init()` method
    start.call(&[])?;

    let tui = setup_tui()?;

    loop {
        let (cols, rows) = terminal::size()?;
        draw.call(&[Value::I32(rows as i32), Value::I32(cols as i32)])?;

        // FIXME: This downcasting mess needs to be abstracted away
        let mut state = wasi_env.state();
        let wasi_file = state.fs.stdout_mut()?.as_mut().unwrap();
        let output: &mut fluff::Pipe = wasi_file.downcast_mut().unwrap();
        // Needed because raw mode doesn't implicitly return to the start of the line
        write!(
            io::stdout(),
            "{}\n\r",
            output.to_string().lines().collect::<Vec<_>>().join("\n\r")
        )?;
        output.clear();

        let wasi_file = state.fs.stdin_mut()?.as_mut().unwrap();
        let input: &mut fluff::Pipe = wasi_file.downcast_mut().unwrap();
        input.clear();

        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => break,
            Event::Key(e) => {
                writeln!(input, "{}\r", serde_json::to_string(&e)?)?;
                drop(state);
                // Need to release the implicit `state` mutex or I deadlock!
                handle_key.call(&[])?;
            }
            _ => (),
        }
    }

    teardown_tui(tui)?;
    Ok(())
}

pub type TUI = Terminal<CrosstermBackend<Stdout>>;

pub fn setup_tui() -> Result<TUI, Box<dyn Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut tui = Terminal::new(backend)?;
    tui.hide_cursor()?;
    Ok(tui)
}

pub fn teardown_tui(mut tui: TUI) -> Result<(), Box<dyn Error>> {
    terminal::disable_raw_mode()?;
    let stdout = tui.backend_mut();
    stdout.execute(terminal::LeaveAlternateScreen)?;
    tui.show_cursor()?;
    Ok(())
}

fn host_open_file(arc_state: &mut Arc<Mutex<WasiState>>) {
    let mut state = arc_state.lock().unwrap();
    let wasi_file = state.fs.stdout_mut().unwrap().as_mut().unwrap();
    let output: &mut fluff::Pipe = wasi_file.downcast_mut().unwrap();
    Command::new("xdg-open")
        .arg(format!(
            "{}/{}",
            ROOT_PATH,
            output.to_string().lines().next().unwrap()
        ))
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    output.clear();
}
