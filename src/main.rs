extern crate hostname;
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
    // TODO: Factor out more helper functions
    // TODO: Change state.variables to a wrapper around a hashmap that updates sys env vars, and can take String and str
    // TODO: Read some config file to get things like the home directory
    // TODO: Semicolons between commands on a single line
    // TODO: Pipes and output redirection
    // TODO: Syntax highlighting
    let mut shell = ShellState {
        directory: PathBuf::new(),
        user: "ben".to_owned(),
        hostname: get_hostname().unwrap(),
        variables: HashMap::new(),
        history: CircularBuffer::new(10000),
    };

    // Load and configure environment variables
    for (key, value) in env::vars_os() {
        shell.variables.insert(key, value);
    }
    shell.variables.insert(OsString::from("SHELL"), OsString::from("rsh"));

    let home = shell.variables[OsStr::new("HOME")].to_owned();
    let hist_loc = Path::new(&home).join(Path::new(OsStr::new(".rsh_history")));
    shell.variables.insert(OsString::from("HISTFILE"), hist_loc.as_os_str().to_owned());
    shell.variables.insert(OsString::from("HISTSIZE"), OsString::from("10000"));
    shell.variables.insert(OsString::from("HOSTNAME"), OsString::from(get_hostname().unwrap()));

    shell.variables.insert(OsString::from("PROMPT"), OsString::from("{BOLD}{WHITE}╭{RED} ➜ {GREEN}{$USER}@{$HOSTNAME}:{CYAN}{$PWD}{WHITE}\n╰ ➤ "));

    shell.directory = Path::new(&shell.variables[&std::ffi::OsString::from("HOME")]).to_path_buf();

    shell.load_history();

    let mut input_buffer = String::new();
    loop {
        shell.prompt_read(&mut input_buffer);
        let mut args = input_buffer.split_whitespace();
        let cmd = args.next().unwrap_or("").to_owned();

        match cmd.as_ref() {
            "" => print!(""),
            "cd" => shell.cd(args),
            "echo" => shell.echo(args),
            "exit" => {
                io::stdout().flush().unwrap();
                shell.save_history();
                std::process::exit(0)
            },
            "history" => shell.print_history(),
            _ => shell.run_command(&cmd, args)
        };
    }
}
