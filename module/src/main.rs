use std::{fs::{self, DirEntry}, cell::RefCell, path::PathBuf, cmp::min};
use colored::*;
use mosaic_plugin::{get_key, KeyCode, open_file};

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

#[derive(Default)]
struct State {
    path: PathBuf,
    files: Vec<DirEntry>,
    selected: usize,
}

// FIXME: Consider ditching the main function and just using init()
fn main() {
    refresh_directory();
}

#[no_mangle]
pub fn draw(rows: i32, cols: i32) {
    STATE.with(|state| {
        let state = state.borrow_mut();
        for i in 0..rows as usize - 1 {
            if let Some(entry) = state.files.get(i) {
                let mut path = entry.path().to_string_lossy().into_owned().normal();
                if entry.file_type().unwrap().is_dir() {
                    path = path.dimmed().bold();
                }

                if i == state.selected {
                    println!("{}", path.reversed());
                } else {
                    println!("{}", path);
                }
            } else {
                println!();
            }
        }
    });
}

#[no_mangle]
pub fn handle_key() {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        match get_key().code {
            KeyCode::Up => {
                state.selected = state.selected.saturating_sub(1);
            }
            KeyCode::Down => {
                let next = state.selected.saturating_add(1);
                state.selected = min(state.files.len() - 1, next);
            }
            KeyCode::Right => {
                let path = &state.files[state.selected].path();
                let entry = &state.files[state.selected];
                if entry.file_type().unwrap().is_dir() {
                    state.path = path.clone();
                    state.selected = 0;
                    drop(state);
                    refresh_directory();
                } else {
                    open_file(path);
                }
            }
            KeyCode::Left => {
                state.path.pop();
                state.selected = 0;
                drop(state);
                refresh_directory();
            }
            _ => (),
        };
    });
}

fn refresh_directory() {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.files = fs::read_dir(&state.path).unwrap()
        .filter_map(|res| res.ok())
        .collect();

        state.files.sort_by_key(DirEntry::path);
    });
}