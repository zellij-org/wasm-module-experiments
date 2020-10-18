use std::io;

use wasmer::{Exports, Function, Instance, Module, Store};
use wasmer_compiler_llvm::LLVM;
use wasmer_engine_jit::JIT;
use wasmer_wasi::WasiState;

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
    let store = Store::new(&JIT::new(&LLVM::default()).engine());

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
        // .env("KEY", "Value")
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

    // Check for output
    let state = wasi_env.state();
    let wasi_file = state.fs.stdout().unwrap().as_ref().unwrap();
    let output: &fluff::OutputCapturer = wasi_file.downcast_ref().unwrap();
    println!("{}", output);
    println!("\nAgain! But backwards for giggles!\n");
    println!("{}", output.to_string().chars().rev().collect::<String>());

    Ok(())
}