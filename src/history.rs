extern crate std;

use std::io::Write;
use std::io::BufRead;
use std::ffi::OsStr;
use state::ShellState;
use circular_buffer::CircularBuffer;

impl ShellState {
    pub fn load_history(&mut self) {
        let histsize = self.variables[OsStr::new("HISTSIZE")].to_string_lossy().parse::<usize>().unwrap_or(10000);
        self.history = CircularBuffer::new(histsize);

        if let Ok(file) = std::fs::File::open(&self.variables[OsStr::new("HISTFILE")]) {
            let reader = std::io::BufReader::new(file);
            for line in reader.lines().filter_map(|l| l.ok()) {
                self.history.push(line);
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
        if let Ok(mut file) = std::fs::File::create(&self.variables[OsStr::new("HISTFILE")]) {
            for entry in self.history.iter() {
                file.write_all(entry.as_bytes());
                file.write_all(b"\n");
            }
        }
    }
}