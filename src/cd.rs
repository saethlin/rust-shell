extern crate std;

use std::path::Path;
use state::ShellState;

impl ShellState {
    pub fn cd(&mut self, mut args: std::str::SplitWhitespace) {
        #![allow(unused)]
        match args.next() {
            Some(dir) => {
                let path = Path::new(dir);
                if path.has_root() && path.is_dir() {
                    self.variables.insert("PWD", path.to_string_lossy().as_ref());
                    std::env::set_current_dir(path);
                    return;
                }
                let proposed_path = Path::new(self.variables.get("PWD").unwrap()).join(path);
                match proposed_path.canonicalize() {
                    Ok(new_path) => {
                        if new_path.as_path().is_dir() {
                            self.variables.insert("PWD", new_path.to_string_lossy().as_ref());
                            std::env::set_current_dir(new_path);
                        } else {
                            println!("{} is a valid path but not a directory", new_path.to_string_lossy());
                        }
                    }
                    Err(..) => println!("path not found: {}", proposed_path.to_string_lossy())
                }
            },
            None => {
                let home_dir = self.variables.get("HOME").unwrap().clone();
                self.variables.insert("PWD", home_dir.to_string_lossy().as_ref());
                std::env::set_current_dir(Path::new(&home_dir));
            }
        }
    }
}