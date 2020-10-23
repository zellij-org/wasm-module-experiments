use std::{fs, io};
use colored::*;

#[link(wasm_import_module = "mosaic")]
extern {
    fn magic_number() -> i32;
}
fn main() -> io::Result<()> {
    let mut entries = fs::read_dir(".")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    // The order in which `read_dir` returns entries is not guaranteed. If reproducible
    // ordering is required the entries should be explicitly sorted.

    entries.sort();

    // The entries have now been sorted by their path.

    println!("{:?}", entries);

    println!("Getting brave and calling a foreign function!");
    let magic = unsafe { magic_number() };
    println!("The magic number was: {:?}", magic);

    println!("{} {} !", "it".green(), "works".blue().bold());

    Ok(())
}