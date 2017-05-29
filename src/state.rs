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
use self::termios::{Termios, TCSANOW, tcsetattr};
use circular_buffer::CircularBuffer;

pub struct ShellState {
    pub directory: PathBuf,
    pub user: String,
    pub hostname: String,
    pub variables: HashMap<std::ffi::OsString, std::ffi::OsString>,
    pub history: CircularBuffer<String>,
}

fn print_buffer(buf: &str, clear: bool) {
    if clear {
        print!("\r    ");
        print!("{}", " ".repeat(buf.len()+1));
    }
    print!("\r╰ ➤ {}", buf);
}

// "\r{BOLD}{BRIGHT_WHITE}╭{BRIGHT_RED} ➜ {BRIGHT_GREEN}{$USER}@{$HOSTNAME}:{BRIGHT_CYAN}{$PWD}{BRIGHT_WHITE}\n\r╰ ➤ "


impl ShellState {

    pub fn print_prompt(&self) {
        #![allow(unused)]
        use std::ffi::{OsStr, OsString};
        let mut t = term::stdout().unwrap();
        let mut buf = String::new();
        let prompt = OsString::from("{BOLD}{BRIGHT_WHITE}╭{BRIGHT_RED} ➜ {BRIGHT_GREEN}{$USER}@{$HOST}:{BRIGHT_CYAN}{$PWD}{BRIGHT_WHITE}\n╰ ➤ ");
        let mut in_braces = true;
        print!("\r");
        for c in prompt.to_string_lossy().chars() {
            match c {
                '{' => {
                    in_braces = true;
                }
                '}' => {
                    if buf.starts_with('$') {
                        let (_, key) = buf.split_at(1);
                        if let Some(val) = self.variables.get(OsStr::new(key)) {
                            print!("{}", val.to_string_lossy());
                        }
                    }
                    else {
                        match buf.as_ref() {
                            "BOLD" => {t.attr(term::Attr::Bold);}
                            "BRIGHT_WHITE" => {t.fg(term::color::BRIGHT_WHITE);}
                            "BRIGHT_RED" => {t.fg(term::color::BRIGHT_RED);}
                            "BRIGHT_GREEN" => {t.fg(term::color::BRIGHT_GREEN);}
                            "BRIGHT_CYAN" => {t.fg(term::color::BRIGHT_CYAN);}
                            _ => {},
                        };
                    }
                    in_braces = false;
                    buf.clear();

                }
                '\n' => {
                    print!("\n\r");
                }
                _ => {
                    if in_braces {
                        buf.push(c);
                    }
                    else {
                        print!("{}", c)
                    }
                }
            }
        }
        t.reset().unwrap();
        io::stdout().flush().unwrap();
    }

    pub fn prompt(&self) {
        #![allow(unused)]

        self.print_prompt();
        return;

        let mut t = term::stdout().unwrap();

        t.attr(term::Attr::Bold);
        t.fg(term::color::BRIGHT_WHITE);
        write!(t, "\r╭").unwrap();
        t.fg(term::color::BRIGHT_RED);
        write!(t, " ➜ ").unwrap();
        t.fg(term::color::BRIGHT_GREEN);
        write!(t, "{}@{}:", self.user, self.hostname).unwrap();
        t.fg(term::color::BRIGHT_CYAN);
        write!(t, "{}", self.directory.to_string_lossy()).unwrap();
        t.fg(term::color::BRIGHT_WHITE);
        write!(t, "\n\r╰ ➤ ").unwrap();
        t.reset().unwrap();

        io::stdout().flush().unwrap(); // Flush to ensure stdout is printed immediately
    }

    pub fn prompt_read(&mut self, input_buffer: &mut String) {
        let stdin = 0;
        let old_term = Termios::from_fd(stdin).unwrap();
        let mut term = Termios::from_fd(stdin).unwrap();
        termios::cfmakeraw(&mut term);
        tcsetattr(stdin, TCSANOW, &term).unwrap();

        self.prompt();
        input_buffer.clear();
        let mut charbuf = [0; 1];
        let mut suggestion = String::new();
        let mut cursor_position = 0;

        loop {
            io::stdin().read_exact(&mut charbuf).unwrap();

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
                        self.save_history();
                        std::process::exit(0);
                    }
                }
                // tab inserts the current autocomplete suggestion
                9 => {
                    if !suggestion.is_empty() {
                        input_buffer.clear();
                        input_buffer.push_str(suggestion.as_str());
                        print_buffer(input_buffer, true);
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
                    io::stdin().read_exact(&mut charbuf).unwrap();
                    io::stdin().read_exact(&mut charbuf).unwrap();
                    match charbuf[0] {
                        // Up access last entry
                        65 => {
                            if input_buffer.is_empty() {
                                input_buffer.clear();
                                input_buffer.push_str(self.history.head().unwrap_or(&"".to_owned()));
                                print_buffer(input_buffer, true);
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
                    print_buffer(input_buffer, false);
                    cursor_position += 1;
                },
            }
            io::stdout().flush().unwrap(); // Always flush after getting input
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
