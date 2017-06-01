extern crate hostname;
extern crate glob;
extern crate rust_shell as shell;

use std::io;
use std::io::Write;
use std::str;
use hostname::get_hostname;
use shell::state::ShellState;
use shell::circular_buffer::CircularBuffer;
use shell::envars::Envars;

fn main() {
    // TODO: Factor out more helper functions
    // TODO: Semicolons between commands on a single line
    // TODO: Pipes and output redirection
    // TODO: Syntax highlighting
    let mut shell = ShellState {
        variables: Envars::load(),
        history: CircularBuffer::new(10000),
    };

    // login only gives me $HOME, $SHELL, $PATH, $LOGNAME, and $MAIL, so provide defaults here
    shell.variables.insert("PROMPT", "PROMPT={BOLD}{WHITE}╭{RED} ➜ {GREEN}{$USER}@{$HOSTNAME}:{CYAN}{$PWD}{WHITE}\n╰ ➤ ");
    shell.variables.insert("HISTSIZE", "1000");
    shell.variables.insert("USER", "ben");
    shell.variables.insert("HOSTNAME", get_hostname().unwrap().as_ref());

    shell.read_config();

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
