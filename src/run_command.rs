extern crate std;
extern crate hostname;
extern crate glob;

use std::str;
use std::ffi::OsString; // Probably want OsStr in a few places
use std::path::Path;
use std::process::Command;
use std::fs;
use state::ShellState;

impl ShellState {
    pub fn run_command(&self, command: &str, args: std::str::SplitWhitespace) {

        // Very crude glob support
        let mut expanded_args = Vec::new();
        for arg in args {
            if !arg.contains('*') {
                expanded_args.push(OsString::from(arg));
                continue;
            }

            let mut pattern = self.variables.get("PWD").unwrap().clone();
            pattern.push(arg);
            match glob::glob(pattern.to_str().unwrap()) {
                Ok(result_iter) => {
                    for entry in result_iter.filter_map(|e| e.ok()) {
                        expanded_args.push(entry.as_os_str().to_owned());
                    }
                }
                Err(..) => expanded_args.push(OsString::from(arg)),
            }
        }

        if command == "ls" || command == "grep" {
            expanded_args.push(OsString::from("--color=auto"));
        }

        if Path::new(command).is_file() {
            match Command::new(Path::new(command))
                .args(expanded_args)
                .current_dir(self.variables.get("PWD").unwrap().clone())
                .spawn() {
                Ok(mut child) => {
                    child.wait().unwrap();
                    ()
                } // This should be an unwrap_or_else
                Err(_) => println!("command failed to launch: {}", command),
            };
            return;
        }

        let path = self.variables.get("PATH").unwrap().clone();

        for entries in path.into_string()
            .unwrap()
            .split(':')
            .map(|dir| fs::read_dir(Path::new(dir)))
            .filter_map(|e| e.ok())
        {
            // loop over the iterator of every directory in PATH that's possible to read
            for dir_entry in entries
                .filter_map(|e| e.ok()) // Only entries that are ok
                .filter(|e| &e.file_name() == command)
            {
                // Check if entry filename matches
                match Command::new(dir_entry.path())
                    .args(expanded_args)
                    .current_dir(self.variables.get("PWD").unwrap().clone())
                    .spawn() {
                    Ok(mut child) => {
                        child.wait().unwrap();
                        ()
                    } // This should be an unwrap_or_else
                    Err(_) => println!("command failed to launch: {}", command),
                };
                return;
            }
        }
        println!("command not found: {}", command);
    }
}
