extern crate std;

use std::path::Path;
use std::ffi::OsString;
use std::env;
use state::ShellState;

pub fn exec(state: &mut ShellState, args: &mut std::str::SplitWhitespace) {
    match args.next() {
        Some(dir) => {
            let path = Path::new(dir);
            if path.has_root() && path.is_dir() {
                state.directory = path.to_owned();
                std::env::set_current_dir(state.directory.as_path());
                return;
            }
            let proposed_path = state.directory.join(path);
            match proposed_path.canonicalize() {
                Ok(new_path) => {
                    if new_path.as_path().is_dir() {
                        state.directory = new_path;
                        std::env::set_current_dir(state.directory.as_path());
                    } else {
                        println!("{} is a valid path but not a directory", new_path.to_string_lossy());
                    }
                }
                Err(..) => println!("path not found: {}", proposed_path.to_string_lossy())
            }
        },
        None => {
            state.directory = Path::new(&state.variables[&OsString::from("HOME")]).to_owned()
        }
    }
}
