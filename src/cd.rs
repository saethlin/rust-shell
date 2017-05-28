extern crate std;

use std::path::Path;
use std::ffi::OsString;
use state::ShellState;

impl ShellState {
    pub fn cd(&mut self, mut args: std::str::SplitWhitespace) {
        #![allow(unused)]
        match args.next() {
            Some(dir) => {
                let path = Path::new(dir);
                if path.has_root() && path.is_dir() {
                    self.directory = path.to_owned();
                    std::env::set_current_dir(self.directory.as_path());
                    return;
                }
                let proposed_path = self.directory.join(path);
                match proposed_path.canonicalize() {
                    Ok(new_path) => {
                        if new_path.as_path().is_dir() {
                            self.directory = new_path;
                            std::env::set_current_dir(self.directory.as_path());
                        } else {
                            println!("{} is a valid path but not a directory", new_path.to_string_lossy());
                        }
                    }
                    Err(..) => println!("path not found: {}", proposed_path.to_string_lossy())
                }
            },
            None => {
                self.directory = Path::new(&self.variables[&OsString::from("HOME")]).to_owned()
            }
        }
    }
}