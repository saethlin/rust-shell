extern crate rust_shell as shell;
extern crate hostname;
extern crate term;

use std::io;
use std::io::Write;
use std::str;
use hostname::get_hostname;
use std::path::{PathBuf, Path};
use std::collections::HashMap;
use shell::commands;
use shell::state::ShellState;

fn main() {
    // TODO: Tab completion
    // TODO: Syntax highlighting
    // TODO: Fix ls column number detection
    // TODO: Add colors to ls output
    // TODO: Read some config file to get things like the home directory
    // TODO: Semicolons between command on a single line
    // TODO: Pipes and output redirection
    let mut state = ShellState {
        directory: PathBuf::new(),
        user: "ben".to_owned(),
        hostname: get_hostname().unwrap(),
        variables: HashMap::new(),
        input_buffer: String::new(),
        output_buffer: String::new(),
    };
    state.variables.insert("HOME".to_owned(), "/home/ben".to_owned());
    state.variables.insert("PATH".to_owned(), "/usr/bin:/bin:".to_owned());
    state.variables.insert("SHELL".to_owned(), "rsh".to_owned());
    state.directory = Path::new(&state.variables["HOME"]).to_path_buf();

    loop {
        let mut args = read(&mut state);
        let cmd = args.next().unwrap_or("");
        let args = args;

        match cmd.as_ref() {
            "" => print!(""),
            "cd" | "dir" => commands::cd::exec(&mut state, args),
            //"cp" => commands::cp::exec(&state, args),
            //"echo" => commands::echo::exec(&state, args),
            //"grep" => commands::grep::exec(&state, args),
            //"ls" => commands::ls::exec(&state, args),
            //"mkdir"
            //"mv"
            //"rm" => commands::rm::exec(&state, args),
            //"touch" => commands::rm::exec(&state, args),
            _ => run_command(&state, &cmd, args)
        };
    }
}

fn prompt(state: &ShellState) {
    #![allow(unused)]
    let mut t = term::stdout().unwrap();

    t.fg(term::color::BRIGHT_WHITE);
    write!(t, "╭").unwrap();
    t.fg(term::color::BRIGHT_RED);
    t.attr(term::Attr::Bold);
    write!(t, " ➜ ").unwrap();
    t.fg(term::color::BRIGHT_GREEN);
    write!(t, "{}@{}:", state.user, state.hostname).unwrap();
    t.fg(term::color::BRIGHT_CYAN);
    write!(t, "{}", state.directory.to_string_lossy()).unwrap();
    t.fg(term::color::BRIGHT_WHITE);
    t.attr(term::Attr::Bold);
    write!(t, "\n╰ ➤ ").unwrap();
    t.reset().unwrap();

    io::stdout().flush().unwrap(); // Flush to ensure stdout is printed immediately
}

fn read(state: &mut ShellState) -> std::str::SplitWhitespace {
    prompt(state);

    state.input_buffer.clear();
    if let Ok(status) = io::stdin().read_line(&mut state.input_buffer) {
        if status == 0 {
            print!("\r\n");
            io::stdout().flush().unwrap();
            std::process::exit(0);
        }

        return state.input_buffer.split_whitespace();
    }
    else {
        return "".split_whitespace()
    }
}

fn run_command(state: &ShellState, command: &str, args: std::str::SplitWhitespace) {
    use std::process::Command;
    use std::fs;

    for entries in state.variables["PATH"].split(':')
        .map(|dir| fs::read_dir(Path::new(dir)))
        .filter_map(|e| e.ok()) { // loop over the iterator of every directory in PATH that's possible to read
        for dir_entry in entries
            .filter_map(|e| e.ok()) // Only entries that are ok
            .filter(|e| &e.file_name() == command) { // Check if entry filename matches
            match Command::new(dir_entry.path())
                .args(args)
                .current_dir(state.directory.clone())
                .spawn() {
                Ok(mut child) => {child.wait().unwrap_or_else(|e| println!("command failed to launch: {}", command); 0)},
                Err(e) => {println!("command failed to launch: {}", command); 0},
            };
            return;
        }
    }
    println!("command not found: {}", command);
}
