use serde::{Deserialize, Serialize};
use std::{io, path::Path};

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub bits: u8,
    //pub shift: bool,
    //pub ctrl: bool,
    //pub alt: bool,
}

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Null,
    Esc,
}

pub fn get_key() -> KeyEvent {
    let mut json = String::new();
    io::stdin().read_line(&mut json).unwrap();
    serde_json::from_str(&json).unwrap()
}

pub fn open_file(path: &Path) {
    println!("{}", path.to_string_lossy());
    unsafe { host_open_file() };
}

#[link(wasm_import_module = "mosaic")]
extern "C" {
    fn host_open_file();
}
