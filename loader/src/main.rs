use wasmer::{Exports, Function, Instance, Module, Store};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_engine_jit::JIT;
use wasmer_wasi::WasiState;
use crossterm::{terminal, ExecutableCommand};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::{
    error::Error,
    io::{self, Stdout, Write},
};
use tui::{backend::CrosstermBackend, Terminal, text::Text, widgets::Paragraph};


mod fluff;

// FIXME: PR to write an ImportObject merging method
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Let's pick a WASM file to load!
    let paths = vec!["target/wasm32-wasi/debug/module.wasm",
                                "asmscript/build/index.wasm",
                                "wapm_packages/_/cowsay@0.2.0/target/wasm32-wasi/release/cowsay.wasm"
                                ];
    println!("\n\nWhich WASM file would you like to load?");
    for (i, path) in paths.iter().enumerate() {
        println!("{}) {}", i + 1, path);
    }
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice: usize = choice.trim().parse()?;

    let wasm_bytes = std::fs::read(paths[choice - 1])?;

    // Create a Store.
    // Note that we don't need to specify the engine/compiler if we want to use
    // the default provided by Wasmer.
    // You can use `Store::default()` for that.
    let store = Store::new(&JIT::new(&Singlepass::default()).engine());

    println!("Compiling module...");
    // Let's compile the Wasm module.
    let module = Module::new(&store, wasm_bytes)?;

    println!("Creating `WasiEnv`...");
    // A place to store captured output
    let output = fluff::OutputCapturer::new();
    // First, we create the `WasiEnv`
    // use std::env;
    // let (l, c) = (env::var("LINES")?, env::var("COLUMNS")?);
    let mut wasi_env = WasiState::new("hello")
        .args(&["These are words of wisdom coming from the mighty Mosaic!"])
        .env("CLICOLOR_FORCE", "1")
        .preopen_dir(".")?
        .stdout(Box::new(output))
        .finalize()?;

    println!("Instantiating module with WASI + host imports...");

    // Then, we get the import object related to our WASI
    // and merge it with our host exports
    let mut import_object = wasi_env.import_object(&module)?;
    let mut host_exports = Exports::new();
    host_exports.insert("magic_number", Function::new_native(&store, || 42));
    import_object.register("mosaic", host_exports);
    let instance = Instance::new(&module, &import_object)?;

    // WASI requires to explicitly set the memory for the `WasiEnv`
    wasi_env.set_memory(instance.exports.get_memory("memory")?.clone());

    println!("Call WASI `_start` function...\n\n");
    // And we just call the `_start` function!
    let start = instance.exports.get_function("_start")?;
    start.call(&[])?;

    let mut tui = setup_tui()?;

    loop {
        // Check for output
        let mut state = wasi_env.state();
        let wasi_file = state.fs.stdout_mut().unwrap().as_mut().unwrap();
        let output: &mut fluff::OutputCapturer = wasi_file.downcast_mut().unwrap();
        write!(io::stdout(), "{}\n\r", output.to_string().lines().collect::<Vec<_>>().join("\n\r"))?;
        if let Event::Key(KeyEvent { code: KeyCode::Char('q'), ..}) = event::read()? {
            break;
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
