extern crate std;
extern crate term;
extern crate termios;

use std::collections::HashMap;
use std::path::PathBuf;
use std::iter;
use std::str;
use std::io;
use std::cmp::max;
use std::io::{Write, Read};
use self::termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};
//use circular_buffer;

pub struct ShellState {
    pub directory: PathBuf,
    pub user: String,
    pub hostname: String,
    pub variables: HashMap<std::ffi::OsString, std::ffi::OsString>,
    pub history: Vec<String>,
}

fn print_buffer(handle: &mut std::io::StdoutLock, buf: &str, clear: bool) {
    if clear {
        print!("\r    ");
        print!("{}", " ".repeat(buf.len()+1));
    }
    print!("\r╰ ➤ {}", buf);
    handle.flush().unwrap();
}



impl ShellState {
    pub fn prompt(&self) {
        #![allow(unused)]
        let mut t = term::stdout().unwrap();

        t.fg(term::color::BRIGHT_WHITE);
        write!(t, "\r╭").unwrap();
        t.fg(term::color::BRIGHT_RED);
        t.attr(term::Attr::Bold);
        write!(t, " ➜ ").unwrap();
        t.fg(term::color::BRIGHT_GREEN);
        write!(t, "{}@{}:", self.user, self.hostname).unwrap();
        t.fg(term::color::BRIGHT_CYAN);
        write!(t, "{}", self.directory.to_string_lossy()).unwrap();
        t.fg(term::color::BRIGHT_WHITE);
        t.attr(term::Attr::Bold);
        write!(t, "\n\r╰ ➤ ").unwrap();
        t.reset().unwrap();

        io::stdout().flush().unwrap(); // Flush to ensure stdout is printed immediately
    }

    pub fn read(&mut self, input_buffer: &mut String) {

        input_buffer.clear();
        let stdout = io::stdout(); // Consider locking this and writing directly
        let stdin = 0;
        let old_term = Termios::from_fd(stdin).unwrap();
        let mut term = Termios::from_fd(stdin).unwrap();
        termios::cfmakeraw(&mut term);
        term.c_lflag &= !(ICANON | ECHO); // what are canonical and echo mode
        tcsetattr(stdin, TCSANOW, &term).unwrap();
        let mut reader = io::stdin();
        let mut charbuf = [0; 1];
        let mut out_handle = stdout.lock();

        let mut suggestion = String::new();

        let mut cursor_position = 0;

        loop {
            reader.read_exact(&mut charbuf).unwrap();

            match charbuf[0] {
                // clear line on a ctrl+c
                3 => {
                    input_buffer.clear();
                    println!();
                    self.prompt();
                },
                // ctrl+d should close the shell, only if the input buffer is empty
                4 => {
                    if input_buffer.is_empty() {
                        println!('\r');
                        io::stdout().flush().unwrap();
                        tcsetattr(stdin, TCSANOW, &old_term).unwrap();
                        std::process::exit(0);
                    }
                }
                // tab inserts the current autocomplete suggestion
                9 => {
                    input_buffer.clear();
                    input_buffer.push_str(suggestion.as_str());
                    print_buffer(&mut out_handle, input_buffer, true);
                    cursor_position = input_buffer.len();
                }
                // return should append the command to history and return to the caller
                13 => {
                    println!("\r");
                    self.history.push(input_buffer.to_owned());
                    tcsetattr(stdin, TCSANOW, &old_term).unwrap();
                    return;
                },
                // Escape character indicates an arrow key
                27 => {
                    reader.read_exact(&mut charbuf).unwrap();
                    reader.read_exact(&mut charbuf).unwrap();
                    match charbuf[0] {
                        // Up access last entry
                        65 => {
                            if input_buffer.is_empty() {
                                input_buffer.clear();
                                input_buffer.push_str(self.history.last().unwrap_or(&"".to_owned()));
                                print_buffer(&mut out_handle, input_buffer, true);
                                cursor_position = input_buffer.len();
                            }
                        },
                        66 => { // Down
                        },
                        67 => { // Right
                            if cursor_position < input_buffer.len() {
                                cursor_position += 1;
                                print!("\r╰ ➤ ");
                                for c in input_buffer.chars().chain(iter::once(' ')).take(cursor_position) {
                                    print!("{}", c);
                                }
                            }
                        }
                        68 => { // Left
                            if cursor_position > 0 {
                                cursor_position -= 1;
                                print!("\r╰ ➤ ");
                                for c in input_buffer.chars().take(cursor_position) {
                                    print!("{}", c);
                                }
                            }
                        }
                        _ => {},
                    }
                    out_handle.flush().unwrap();
                }
                // del, which is the same char as a tilde. // TODO: figure out how to detect modifier keys
                126 => {
                    if !input_buffer.is_empty() && cursor_position < input_buffer.len() {

                        // Purge any currently printed suggestion
                        if !suggestion.is_empty() {
                            print!("\r╰ ➤ {}", " ".repeat(suggestion.len()));
                        }

                        input_buffer.remove(cursor_position);
                        print!("\r╰ ➤ {} \r╰ ➤ ", input_buffer);
                        for c in input_buffer.chars().take(cursor_position) {
                            print!("{}", c);
                        }
                        out_handle.flush().unwrap();
                    }
                }
                // backspace removes one character from the buffer
                127 => {
                    if cursor_position > 0 { // And input_buffer is not empty, but that should be enforced by the other rules on the cursor

                        // Purge any currently printed suggestion
                        if !suggestion.is_empty() {
                            print!("\r╰ ➤ {}", " ".repeat(suggestion.len()));
                        }

                        input_buffer.remove(cursor_position-1);
                        cursor_position -= 1;
                        print!("\r╰ ➤ {} \r╰ ➤ ", input_buffer);
                        for c in input_buffer.chars().take(cursor_position) {
                            print!("{}", c);
                        }
                        out_handle.flush().unwrap();
                    }
                }

                // Everything else is a printable symbol and gets added to the input buffer
                _ => {
                    input_buffer.push(charbuf[0] as char);

                    suggestion.clear();
                    // Find our new suggestion and print it in gray
                    for entry in self.history.iter().rev() {
                        if entry.starts_with(input_buffer.as_str()) {
                            suggestion.push_str(entry);
                            break;
                        }
                    }

                    let mut t = term::stdout().unwrap();
                    t.fg(term::color::MAGENTA).unwrap();
                    write!(t, "\r╰ ➤ {}", suggestion).unwrap();
                    t.fg(term::color::BRIGHT_WHITE).unwrap();

                    print_buffer(&mut out_handle, input_buffer, false);
                    cursor_position += 1;
                },
            }
        }
    }
}

