// As of AssemblyScript 0.10.0, adding `import "wasi"`, will automatically
// import WASI bindings, and add some nice defaults for compiling to WASI.
import "wasi";

// Import Console (for writing to stdout), and FileSystem (for reading/writing files)
// from "as-wasi". An API for working with WASI in AssemblyScript much easier.
import { Console, FileSystem, Descriptor } from "as-wasi";

// Print out hello world!
// This will handle writing to stdout for us using the WASI APIs (e.g fd_write)
Console.log("Hello from AssemblyScript! Look at this cool multi-language support!");

// Import a function from the host and run it
@external("mosaic", "magic_number")
declare function magic_number(): i32;

Console.log("The magic was: " + magic_number().toString());

// Get the contents of the current directory and print them!
let files: string[] | null = FileSystem.readdir("/")
if (files == null) {
  Console.log("Couldn't read the current directory!")
} else {
  let file_list = changetype<string[]>(files);
  Console.log(file_list.toString());
}

/* // We are creating/opening a `helloworld.txt` file
// This code requires the Wasi host to provide a directory on the guest.
// For example, in Wasmtime, if you want to access to the current directory,
// invoke the wasmtime with the flag/argument: `--dir .`
// FileSystem.open will return null if it fails to create/open the file
let filePath: string = "helloworld.txt";
let fileOrNull: Descriptor | null = FileSystem.open(filePath, "w+");

// Check if the FileSystem.open() returned null.
// If fileOrNull is null, that means we could not create/open the file
// (Probably because we did not add the `--dir` flag)
// Throw an error.
if (fileOrNull == null) {
  throw new Error("Could not open the file " + filePath);
}

// Change our type from Descriptor | null, to Descriptor, as we checked above.
// Meaning, we were able to successfully open/create the file
let file = changetype<Descriptor>(fileOrNull);

// Write "Hello World!" to the file.
file.writeStringLn("Hello World!"); */