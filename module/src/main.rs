use std::{fs, io, cell::RefCell};
use colored::*;

thread_local! {
    static COUNTER: RefCell<i32> = RefCell::new(0);
}

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

#[no_mangle]
pub fn handle_key() {
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    COUNTER.with(|counter| {
        println!("{}: {}", counter.borrow(), choice);
        *counter.borrow_mut() += 1;
    });
}