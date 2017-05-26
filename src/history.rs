extern crate std;

use std::io::BufRead;
use std::path::PathBuf;
use std::ffi::OsStr;
use state::ShellState;
use circular_buffer::CircularBuffer;

pub fn load_history(state: &mut ShellState) {
    let path = PathBuf::from(&state.variables[OsStr::new("HISTFILE")]);
    let histsize = state.variables[OsStr::new("HISTSIZE")].to_string_lossy().parse::<usize>().unwrap_or(10000);
    state.history = CircularBuffer::new(histsize);

    let f = std::fs::File::open(path);
    match f {
        Err(..) => {
            println!("Failed to load history file, continuing with blank history");
        }
        Ok(file) => {
            let mut has_read_err = false;
            let reader = std::io::BufReader::new(file);
            for line in reader.lines() {
                match line {
                    Ok(entry) => state.history.push(entry),
                    Err(..) => {
                        if !has_read_err {
                            println!("Error loading history, some entries may be missing");
                            has_read_err = true;
                        }
                    }
                }
            }
        }
    }
}

pub fn print_history(state: &ShellState) {
    for (i, entry) in state.history.iter().enumerate() {
        println!("{:>5.} {}", i, entry);
    }
}
