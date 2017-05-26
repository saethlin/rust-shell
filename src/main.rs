extern crate rust_shell as shell;
extern crate hostname;
extern crate term;

use std::io;
use std::io::Write;
use std::str;
use std::env;
use std::ffi::OsString;
use hostname::get_hostname;
use std::path::{PathBuf, Path};
use std::collections::HashMap;
use shell::state::ShellState;

fn main() {
    // TODO: Interact with system environment variables
    // TODO: Abstract out the prompt and read from a config file
    // TODO: Force-alias ls to color=auto
    // TODO: Syntax highlighting
    // TODO: Read some config file to get things like the home directory
    // TODO: Semicolons between commands on a single line
    // TODO: Pipes and output redirection
    let mut state = ShellState {
        directory: PathBuf::new(),
        user: "ben".to_owned(),
        hostname: get_hostname().unwrap(),
        variables: HashMap::new(),
        history: Vec::new(),
    };

    for (key, value) in env::vars_os() {
        state.variables.insert(key, value);
    }
    state.variables.insert(OsString::from("SHELL"), OsString::from("rsh"));
    state.directory = Path::new(&state.variables[&std::ffi::OsString::from("HOME")]).to_path_buf();

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
            _ => run_command(&state, &cmd, args)
        };
    }
}

fn run_command(state: &ShellState, command: &str, args: std::str::SplitWhitespace) {
    use std::process::Command;
    use std::fs;

    let path = state.variables[&std::ffi::OsString::from("PATH")].clone();

    for entries in path.into_string().unwrap().split(':')
        .map(|dir| fs::read_dir(Path::new(dir)))
        .filter_map(|e| e.ok()) { // loop over the iterator of every directory in PATH that's possible to read
        for dir_entry in entries
            .filter_map(|e| e.ok()) // Only entries that are ok
            .filter(|e| &e.file_name() == command) { // Check if entry filename matches
            match Command::new(dir_entry.path())
                .args(args)
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
