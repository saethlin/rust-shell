extern crate std;

use std::io::Write;
use std::io::BufRead;
use std::path::PathBuf;
use std::ffi::OsStr;
use state::ShellState;
use circular_buffer::CircularBuffer;

impl ShellState {
    pub fn load_history(&mut self) {
        let path = PathBuf::from(&self.variables[OsStr::new("HISTFILE")]);
        let histsize = self.variables[OsStr::new("HISTSIZE")].to_string_lossy().parse::<usize>().unwrap_or(10000);
        self.history = CircularBuffer::new(histsize);

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
                        Ok(entry) => self.history.push(entry),
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

    pub fn print_history(&self) {
        for (i, entry) in self.history.iter().enumerate() {
            println!("{:>5.} {}", i, entry);
        }
    }

    pub fn save_history(&self) {
        #![allow(unused)] // We do this on the way out, so failures are just too bad
        let path = PathBuf::from(&self.variables[OsStr::new("HISTFILE")]);
        let f = std::fs::File::open(path);
        if let Ok(file) = f {
            let mut writer = std::io::BufWriter::new(file);
            for entry in self.history.iter() {
                writer.write(entry.as_bytes());
                writer.write(b"\n");
                println!("{}", entry);
            }
        }
    }
}