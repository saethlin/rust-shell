#![feature(alloc_system)]
extern crate alloc_system;

extern crate hostname;
extern crate term;
extern crate glob;
extern crate rust_shell as shell;


use std::io;
use std::io::Write;
use std::str;
use std::env;
use std::ffi::{OsString, OsStr};
use hostname::get_hostname;
use std::path::{PathBuf, Path};
use std::collections::HashMap;
use shell::state::ShellState;
use shell::circular_buffer::CircularBuffer;

fn main() {
    // TODO: Refactor around a single state struct with helper functions
    // TODO: Persistent history
    // TODO: Interact with system environment variables
    // TODO: Read some config file to get things like the home directory
    // TODO: Semicolons between commands on a single line
    // TODO: Pipes and output redirection
    // TODO: Syntax highlighting
    let mut state = ShellState {
        directory: PathBuf::new(),
        user: "ben".to_owned(),
        hostname: get_hostname().unwrap(),
        variables: HashMap::new(),
        history: CircularBuffer::new(10000),
    };

    // Load and configure environment variables
    for (key, value) in env::vars_os() {
        state.variables.insert(key, value);
    }
    state.variables.insert(OsString::from("SHELL"), OsString::from("rsh"));

    let home = state.variables[OsStr::new("HOME")].to_owned();
    let hist_loc = Path::new(&home).join(Path::new(OsStr::new(".rsh_history")));
    state.variables.insert(OsString::from("HISTFILE"), hist_loc.as_os_str().to_owned());
    state.variables.insert(OsString::from("HISTSIZE"), OsString::from("10000"));

    state.directory = Path::new(&state.variables[&std::ffi::OsString::from("HOME")]).to_path_buf();

    // Load history
    shell::history::load_history(&mut state);

    let mut input_buffer = String::new();

    loop {
        state.prompt();
        state.read(&mut input_buffer);
        let mut args = input_buffer.split_whitespace();
        let cmd = args.next().unwrap_or("").to_owned();

        match cmd.as_ref() {
            "" => print!(""),
            "cd" => shell::cd::exec(&mut state, &mut args),
            "echo" => shell::echo::exec(&state, &mut args),
            "exit" => {io::stdout().flush().unwrap(); std::process::exit(0)},
            "ls" => {run_command(&state, &cmd, args)},
            "history" => {shell::history::print_history(&state)},
            _ => run_command(&state, &cmd, args)
        };
    }
}

fn run_command(state: &ShellState, command: &str, args: std::str::SplitWhitespace) {
    use std::process::Command;
    use std::fs;

    // Very crude glob support
    let mut expanded_args = Vec::new();
    for arg in args {
        if !arg.contains('*') {
            expanded_args.push(OsString::from(arg));
            continue;
        }

        let mut pattern = state.directory.clone();
        pattern.push(arg);
        match glob::glob(pattern.to_str().unwrap()) {
            Ok(result_iter) => {
                for entry in result_iter.filter_map(|e| e.ok()) {
                    expanded_args.push(entry.as_os_str().to_owned());
                }
            },
            Err(..) => expanded_args.push(OsString::from(arg)),
        }
    }

    if command == "ls" {
        expanded_args.push(OsString::from("--color=auto"));
    }

    if Path::new(command).is_file() {
        match Command::new(Path::new(command))
            .args(expanded_args)
            .current_dir(state.directory.clone())
            .spawn() {
            Ok(mut child) => {child.wait().unwrap(); ()} // This should be an unwrap_or_else
            Err(_) => println!("command failed to launch: {}", command),
        };
        return;
    }

    let path = state.variables[&std::ffi::OsString::from("PATH")].clone();

    for entries in path.into_string().unwrap().split(':')
        .map(|dir| fs::read_dir(Path::new(dir)))
        .filter_map(|e| e.ok()) { // loop over the iterator of every directory in PATH that's possible to read
        for dir_entry in entries
            .filter_map(|e| e.ok()) // Only entries that are ok
            .filter(|e| &e.file_name() == command) { // Check if entry filename matches
            match Command::new(dir_entry.path())
                .args(expanded_args)
                .current_dir(state.directory.clone())
                .spawn() {
                Ok(mut child) => {child.wait().unwrap(); ()} // This should be an unwrap_or_else
                Err(_) => println!("command failed to launch: {}", command),
            };
            return;
        }
    }
    println!("command not found: {}", command);
}
