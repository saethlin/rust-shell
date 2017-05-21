extern crate rust_shell as shell;
extern crate hostname;
extern crate term;

use shell::commands;
use std::io;
use std::process::Command;
use std::str;
use hostname::get_hostname;
use std::path::{PathBuf, Path};
use std::collections::HashMap;
use shell::state::ShellState;

fn main() {
    // TODO: Search PATH for something to run if unknown
    // TODO: Read some config file to get things like the home directory
    // TODO: Tab completion
    // TODO: Syntax highlighting
    // TODO: proper ls formatting
    let mut state = ShellState {
        directory: PathBuf::new(),
        user: "ben".to_string(),
        hostname: get_hostname().unwrap(),
        variables: HashMap::new(),
    };
    state.variables.insert("HOME".to_owned(), "/home/ben".to_owned());
    state.variables.insert("PATH".to_owned(), "/usr/bin:/bin:".to_owned());
    state.directory = Path::new(&state.variables["HOME"]).to_path_buf();

    loop {
        let (cmd, args) = read(&state);

        match cmd.as_ref() {
            "" => print!(""),
            "cd" => commands::cd::exec(&mut state, &args),
            "ls" => commands::ls::exec(&state, args),
            "echo" => commands::echo::exec(&state, args),
            //"rm" => commands::rm::exec(&state, args),
            //"touch" => commands::rm::exec(&state, args),
            //"grep" => commands::grep::exec(&state, args),
            _ => run_command(&state, cmd, &args)
        };
    }
}

fn prompt(state: &ShellState) {
    use std::io::prelude::*;
    let mut t = term::stdout().unwrap();

    t.fg(term::color::BRIGHT_WHITE).unwrap();
    write!(t, "╭").unwrap();
    t.fg(term::color::BRIGHT_RED).unwrap();
    t.attr(term::Attr::Bold).unwrap();
    write!(t, " ➜ ").unwrap();
    t.fg(term::color::BRIGHT_GREEN).unwrap();
    write!(t, "{}@{}:", state.user, state.hostname).unwrap();
    t.fg(term::color::BRIGHT_CYAN).unwrap();
    write!(t, "{}", state.directory.to_string_lossy()).unwrap();
    t.fg(term::color::BRIGHT_WHITE).unwrap();
    t.attr(term::Attr::Bold).unwrap();
    write!(t, "\n╰ ➤ ").unwrap();
    t.reset().unwrap();

    io::stdout().flush().unwrap();    // Flush to ensure stdout is printed immediately
}

fn read(state: &ShellState) -> (String, Vec<String>) {
    prompt(state);
    let mut line = "".to_string();
    io::stdin().read_line(&mut line).unwrap();
    // Last character is a line-break we don't need
    line.pop();

    let params: Vec<String> = line.split(' ').map(|x| x.to_string()).collect();
    let mut iter = params.into_iter();

    let cmd = iter.next().unwrap();
    let rest: Vec<String> = iter.collect();

    (cmd, rest)
}

fn run_command(state: &ShellState, command: String, args: &[String]) -> () {
    println!("External command invoked");
    match Command::new(command)
        .args(args)
        .current_dir(state.directory.clone())
        .output() {
            Ok(out) => {
                println!("{}", str::from_utf8(&out.stdout).unwrap());
            },
            Err(e) => {
                println!("Error: {}", e);
            }
        }
}
