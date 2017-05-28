extern crate std;
extern crate term;
extern crate termios;

use std::collections::HashMap;
use std::path::PathBuf;
use std::iter;
use std::str;
use std::fs;
use std::io;
use std::io::{Write, Read};
use self::termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};
use circular_buffer::CircularBuffer;

pub struct ShellState {
    pub directory: PathBuf,
    pub user: String,
    pub hostname: String,
    pub variables: HashMap<std::ffi::OsString, std::ffi::OsString>,
    pub history: CircularBuffer<String>,
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

    pub fn prompt_read(&mut self, input_buffer: &mut String) {
        self.prompt();

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
                    if !suggestion.is_empty() {
                        input_buffer.clear();
                        input_buffer.push_str(suggestion.as_str());
                        print_buffer(&mut out_handle, input_buffer, true);
                        cursor_position = input_buffer.len();
                    }
                }
                // return/enter should append the command to history and return to the caller
                13 => {
                    // Clear any active suggestion
                    if suggestion.len() > input_buffer.len() {
                        print!("{}", " ".repeat(suggestion.len() - input_buffer.len()));
                    }
                    println!("\r");
                    if self.history.tail().unwrap_or(&"".to_owned()) != input_buffer {
                        self.history.push(input_buffer.to_owned());
                    }
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
                                input_buffer.push_str(self.history.head().unwrap_or(&"".to_owned()));
                                print_buffer(&mut out_handle, input_buffer, true);
                                cursor_position = input_buffer.len();
                            }
                        },
                        // Down
                        //66 => { },
                        // Right
                        67 => {
                            if cursor_position < input_buffer.len() {
                                cursor_position += 1;
                                print!("\r╰ ➤ ");
                                for c in input_buffer.chars().chain(iter::once(' ')).take(cursor_position) {
                                    print!("{}", c);
                                }
                            }
                        }
                        // Left
                        68 => {
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

                    print!("\r╰ ➤ {}", " ".repeat(suggestion.len()));
                    suggestion.clear();

                    // Try to find a suggestion from the contents of the current working directory

                    // Split off the last word in the input buffer
                    // This unwrap is safe because rsplitn always yields at least one element
                    let last_word = input_buffer.as_str().rsplitn(2, ' ').next().unwrap();
                    if !last_word.is_empty() && !last_word.starts_with('-') {
                        match self.find_match_directory(last_word) {
                            Some(dirmatch) => {
                                suggestion.push_str(dirmatch.as_str());
                                let mut t = term::stdout().unwrap();
                                t.fg(term::color::MAGENTA).unwrap();
                                let print_this : String = suggestion.chars().skip(last_word.len()).collect();
                                write!(t, "\r╰ ➤ {}{}", " ".repeat(input_buffer.len()), print_this).unwrap();
                                t.fg(term::color::BRIGHT_WHITE).unwrap();
                            },
                            None => {
                                if let Some(histmatch) = self.find_match_history(input_buffer) {
                                    suggestion.push_str(&histmatch);
                                    let mut t = term::stdout().unwrap();
                                    t.fg(term::color::MAGENTA).unwrap();
                                    write!(t, "\r╰ ➤ {}", suggestion).unwrap();
                                    t.fg(term::color::BRIGHT_WHITE).unwrap();
                                }
                            }
                        }
                    }
                    print_buffer(&mut out_handle, input_buffer, false);
                    cursor_position += 1;
                },
            }
        }
    }

    fn find_match_directory(&self, pattern: &str) -> Option<String> {
        if let Ok(entries) = fs::read_dir(&self.directory) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(str_name) = entry.file_name().into_string() {
                    if str_name.as_str().starts_with(pattern) {
                        return Some(str_name);
                    }
                }
            }
        }
        None
    }

    fn find_match_history(&self, pattern: &str) -> Option<String> {
        for entry in self.history.iter_rev() {
            if entry.starts_with(pattern) && entry != pattern {
                return Some(entry.to_owned());
            }
        }
        None
    }
}
