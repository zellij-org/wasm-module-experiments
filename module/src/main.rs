mod state;

use colored::*;
use mosaic_plugin::register_plugin;
use state::{FsEntry, State};
use std::{cmp::min, fs::read_dir};

register_plugin!(State);

impl MosaicPlugin for State {
    fn init(&mut self) {
        refresh_directory(self);
    }

    fn draw(&mut self, rows: usize, cols: usize) {
        for i in 0..rows as usize - 1 {
            if self.selected() < self.scroll() {
                *self.scroll_mut() = self.selected();
            }
            if self.selected() - self.scroll() + 2 > rows as usize {
                *self.scroll_mut() = self.selected() + 2 - rows as usize;
            }
            let i = self.scroll() + i;
            if let Some(entry) = self.files.get(i) {
                let mut path = entry.as_line(cols as usize).normal();

                if let FsEntry::Dir(..) = entry {
                    path = path.dimmed().bold();
                }

                if i == self.selected() {
                    println!("{}", path.reversed());
                } else {
                    println!("{}", path);
                }
            } else {
                println!();
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                *self.selected_mut() = self.selected().saturating_sub(1);
            }
            KeyCode::Down => {
                let next = self.selected().saturating_add(1);
                *self.selected_mut() = min(self.files.len() - 1, next);
            }
            KeyCode::Right | KeyCode::Enter => match self.files[self.selected()].clone() {
                FsEntry::Dir(p, _) => {
                    self.path = p;
                    refresh_directory(self);
                }
                FsEntry::File(p, _) => open_file(&p),
            },
            KeyCode::Left => {
                self.path.pop();
                refresh_directory(self);
            }
            _ => (),
        };
    }
}

fn refresh_directory(state: &mut State) {
    state.files = read_dir(&state.path)
        .unwrap()
        .filter_map(|res| {
            res.and_then(|d| {
                if d.metadata()?.is_dir() {
                    let children = read_dir(d.path())?.count();
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
}
