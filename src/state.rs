extern crate std;
extern crate termcolor;
extern crate termios;

use std::iter;
use std::str;
use std::fs;
use std::io;
use std::io::{Write, Read};
use std::io::Error;
use self::termios::{Termios, TCSANOW, tcsetattr};
use self::termcolor::{Color, ColorChoice, ColorSpec, WriteColor};
use circular_buffer::CircularBuffer;
use envars::Envars;

pub struct ShellState {
    pub variables: Envars,
    pub history: CircularBuffer<String>,
}

pub struct PromptState {
    pub input_buffer: String,
    pub suggestion: String,
    pub cursor_position: usize,
    pub total_len: usize,
}

impl PromptState {
    pub fn clear_suggestion(&self) {
        print!("{}\r", " ".repeat(self.total_len));
        print!("{}", self.input_buffer);
    }
}

fn getchar_raw() -> Result<u8, Error> {
    let stdin = 0;
    let old_term = Termios::from_fd(stdin)?;
    let mut term = Termios::from_fd(stdin)?;
    termios::cfmakeraw(&mut term);
    tcsetattr(stdin, TCSANOW, &term)?;

    let mut charbuf = [0; 1];
    io::stdin().read_exact(&mut charbuf)?;
    tcsetattr(stdin, TCSANOW, &old_term)?;

    Ok(charbuf[0])
}

impl ShellState {
    pub fn prompt(&self) {
        #![allow(unused)]
        let mut stdout = termcolor::StandardStream::stdout(ColorChoice::Auto);
        let mut spec = termcolor::ColorSpec::new();
        let mut buf = String::new();
        let prompt = self.variables.get("PROMPT").unwrap();
        let mut in_braces = false;
        print!("\r");
        for c in prompt.to_string_lossy().chars() {
            match c {
                '{' => {
                    in_braces = true;
                }
                '}' => {
                    if buf.starts_with('$') {
                        let (_, key) = buf.split_at(1);
                        if let Some(val) = self.variables.get(key) {
                            write!(&mut stdout, "{}", val.to_string_lossy());
                        }
                    } else {
                        match buf.as_ref() {
                            "BOLD" => {
                                spec.set_bold(true);
                                stdout.set_color(&spec);
                            }
                            "WHITE" => {
                                spec.set_fg(Some(Color::White));
                                stdout.set_color(&spec);
                            }
                            "RED" => {
                                spec.set_fg(Some(Color::Red));
                                stdout.set_color(&spec);
                            }
                            "GREEN" => {
                                spec.set_fg(Some(Color::Green));
                                stdout.set_color(&spec);
                            }
                            "CYAN" => {
                                spec.set_fg(Some(Color::Cyan));
                                stdout.set_color(&spec);
                            }
                            _ => {}
                        };
                    }
                    in_braces = false;
                    buf.clear();

                }
                '\n' => {
                    write!(&mut stdout, "\n\r");
                }
                _ => {
                    if in_braces {
                        buf.push(c);
                    } else {
                        write!(&mut stdout, "{}", c);
                    }
                }
            }
        }
        stdout.reset();
        io::stdout().flush().unwrap();
    }

    pub fn prompt_read(&mut self, input_buffer: &mut String) {
        self.prompt();
        input_buffer.clear();
        let mut suggestion = String::new();
        let mut cursor_position = 0;
        let mut total_len = 0;

        loop {
            let c = getchar_raw().unwrap();

            match c {
                // ctrl+c clears suggestion and opens a new line
                3 => {
                    print!("\r╰ ➤ {}", " ".repeat(total_len));
                    print!("\r╰ ➤ {}", input_buffer);
                    input_buffer.clear();
                    self.prompt();
                }
                // ctrl+d should close the shell, only if the input buffer is empty
                4 => {
                    // TODO: Refactor this into an exit function? We do have at least one other way to exit
                    if input_buffer.is_empty() {
                        print!("\n\r");
                        io::stdout().flush().unwrap();
                        self.save_history();
                        std::process::exit(0);
                    }
                }
                // tab inserts the current autocomplete suggestion
                9 => {
                    if !suggestion.is_empty() {
                        if let Some(ind) = input_buffer.as_str().rfind(' ') {
                            input_buffer.truncate(ind + 1);
                        }
                        input_buffer.push_str(suggestion.as_str());
                        print!("\r╰ ➤ {}", input_buffer);
                        cursor_position = input_buffer.len();
                        total_len = input_buffer.len();
                    }
                }
                // return/enter should append the command to history and return to the caller
                13 => {
                    // Clear any active suggestion
                    print!("\r╰ ➤ {}", " ".repeat(total_len));
                    print!("\r╰ ➤ {}", input_buffer);
                    print!("\n\r");
                    // Append to history if not a duplicate of the last entry
                    if self.history.tail().unwrap_or(&"".to_owned()) != input_buffer {
                        self.history.push(input_buffer.to_owned());
                    }
                    return;
                }
                // Escape character indicates an arrow key
                27 => {
                    getchar_raw().unwrap();
                    let c = getchar_raw().unwrap();
                    match c {
                        // Up and down should move through the history but instantly be reset by any other command
                        // Up access last entry
                        65 => {
                            if input_buffer.is_empty() {
                                suggestion.clear();
                                input_buffer.clear();
                                input_buffer.push_str(
                                    self.history.head().unwrap_or(&"".to_owned()),
                                );
                                print!("\r╰ ➤ {}", " ".repeat(total_len));
                                print!("\r╰ ➤ {}", input_buffer);
                                cursor_position = input_buffer.len();
                                total_len = input_buffer.len();
                            }
                        }
                        // Down
                        //66 => { },
                        // Right
                        67 => {
                            if cursor_position < input_buffer.len() {
                                cursor_position += 1;
                                print!("\r╰ ➤ ");
                                for c in input_buffer.chars().chain(iter::once(' ')).take(
                                    cursor_position,
                                )
                                {
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
                        _ => {}
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
                        total_len = input_buffer.len();
                    }
                }
                // backspace removes one character from the buffer
                127 => {
                    if cursor_position > 0 {
                        // And input_buffer is not empty, but that should be enforced by the other rules on the cursor

                        // Purge any currently printed suggestion
                        if !suggestion.is_empty() {
                            print!("\r╰ ➤ {}", " ".repeat(total_len));
                        }

                        input_buffer.remove(cursor_position - 1);
                        cursor_position -= 1;
                        print!("\r╰ ➤ {} \r╰ ➤ ", input_buffer);
                        for c in input_buffer.chars().take(cursor_position) {
                            print!("{}", c);
                        }
                        total_len = input_buffer.len();
                    }
                }

                // Everything else is a printable symbol and gets added to the input buffer
                _ => {
                    input_buffer.push(c as char);

                    // Clear currently entered text
                    print!("\r╰ ➤ {}", " ".repeat(total_len));
                    suggestion.clear();

                    // Try to find a suggestion from the contents of the current working directory

                    // Split off the last word in the input buffer
                    // This unwrap is safe because rsplitn always yields at least one element
                    let last_word = input_buffer.as_str().rsplitn(2, ' ').next().unwrap();
                    if !last_word.is_empty() && !last_word.starts_with('-') {
                        match self.find_match_directory(last_word) {
                            Some(dirmatch) => {
                                suggestion.push_str(dirmatch.as_str());
                                let mut stdout =
                                    termcolor::StandardStream::stdout(termcolor::ColorChoice::Auto);
                                stdout
                                    .set_color(ColorSpec::new().set_fg(Some(Color::Magenta)))
                                    .unwrap();
                                let print_this: String =
                                    suggestion.chars().skip(last_word.len()).collect();
                                write!(
                                    &mut stdout,
                                    "\r╰ ➤ {}{}",
                                    " ".repeat(input_buffer.len()),
                                    print_this
                                ).unwrap();
                                stdout.reset().unwrap();
                                total_len = (input_buffer.len() - last_word.len()) +
                                    suggestion.len();
                            }
                            None => {
                                if let Some(histmatch) = self.find_match_history(input_buffer) {
                                    suggestion.push_str(&histmatch);
                                    let mut stdout = termcolor::StandardStream::stdout(
                                        termcolor::ColorChoice::Auto,
                                    );
                                    stdout
                                        .set_color(ColorSpec::new().set_fg(Some(Color::Magenta)))
                                        .unwrap();
                                    write!(&mut stdout, "\r╰ ➤ {}", suggestion).unwrap();
                                    stdout.reset().unwrap();
                                    total_len = suggestion.len();
                                }
                            }
                        }
                    }
                    if suggestion.is_empty() {
                        total_len += 1;
                    }
                    print!("\r╰ ➤ ");
                    print!("{}", input_buffer);
                    cursor_position += 1;
                }
            }
            io::stdout().flush().unwrap(); // Always flush after getting input
        }
    }

    fn find_match_directory(&self, pattern: &str) -> Option<String> {
        if let Ok(entries) = fs::read_dir(&self.variables.get("PWD").unwrap()) {
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
