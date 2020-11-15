use colored::*;
use mosaic_plugin::{get_key, open_file, KeyCode};
use pretty_bytes::converter as pb;
use std::{cell::RefCell, cmp::min, fs, path::PathBuf, collections::HashMap};

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

#[derive(Default, Debug)]
struct State {
    path: PathBuf,
    files: Vec<FsEntry>,
    cursor_hist: HashMap<PathBuf, (usize, usize)>,
}

impl State {
    fn selected_mut(&mut self) -> &mut usize {
        &mut self.cursor_hist.entry(self.path.clone()).or_default().0
    }
    fn selected(&self) -> usize {
        self.cursor_hist.get(&self.path).unwrap_or(&(0,0)).0
    }
    fn scroll_mut(&mut self) -> &mut usize {
        &mut self.cursor_hist.entry(self.path.clone()).or_default().1
    }
    fn scroll(&self) -> usize {
        self.cursor_hist.get(&self.path).unwrap_or(&(0,0)).1
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
enum FsEntry {
    Dir(PathBuf, usize),
    File(PathBuf, u64),
}

impl FsEntry {
    fn path(&self) -> String {
        let path = match self {
            FsEntry::Dir(p, _) => p,
            FsEntry::File(p, _) => p,
        };
        path.file_name().unwrap().to_string_lossy().into_owned()
    }

    fn as_line(&self, width: usize) -> String {
        let info = match self {
            FsEntry::Dir(_, s) => s.to_string(),
            FsEntry::File(_, s) => pb::convert(*s as f64),
        };
        let space = width - info.len();
        let path = self.path();
        if space - 1 < path.len() {
            [&path[..space - 2], &info].join("~ ")
        } else {
            let padding = " ".repeat(space - path.len());
            [path, padding, info].concat()
        }
    }
}

// FIXME: Consider ditching the main function and just using init()
// FIXME: Moving to init gave me some trouble, I'll kick this down the road
fn main() {
    refresh_directory();
}

#[no_mangle]
pub fn draw(rows: i32, cols: i32) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        for i in 0..rows as usize - 1 {
            if state.selected() < state.scroll() {
                *state.scroll_mut() = state.selected();
            }
            if state.selected() - state.scroll() + 2 > rows as usize {
                *state.scroll_mut() = state.selected() + 2 - rows as usize;
            }
            let i = state.scroll() + i;
            if let Some(entry) = state.files.get(i) {
                let mut path = entry.as_line(cols as usize).normal();
                if let FsEntry::Dir(..) = entry {
                    path = path.dimmed().bold();
                }

                if i == state.selected() {
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
                *state.selected_mut() = state.selected().saturating_sub(1);
            }
            KeyCode::Down => {
                let next = state.selected().saturating_add(1);
                *state.selected_mut() = min(state.files.len() - 1, next);
            }
            KeyCode::Right => match state.files[state.selected()].clone() {
                FsEntry::Dir(p, _) => {
                    state.path = p;
                    drop(state);
                    refresh_directory();
                }
                FsEntry::File(p, _) => open_file(&p),
            },
            KeyCode::Left => {
                state.path.pop();
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
        state.files = fs::read_dir(&state.path)
            .unwrap()
            .filter_map(|res| {
                res.and_then(|d| {
                    if d.metadata()?.is_dir() {
                        let children = fs::read_dir(d.path())?.count();
                        Ok(FsEntry::Dir(d.path(), children))
                    } else {
                        let size = d.metadata()?.len();
                        Ok(FsEntry::File(d.path(), size))
                    }
                })
                .ok()
            })
            .collect();

        state.files.sort_unstable();
    });
}
