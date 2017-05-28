extern crate std;

use std::ffi::OsString;
use state::ShellState;

impl ShellState {
    pub fn echo(&self, args: std::str::SplitWhitespace) {
        let mut peeker = args.peekable();
        loop {
            if let Some(arg) = peeker.next() {
                if arg.starts_with('$') {
                    let (_, key) = arg.split_at(1);
                    if let Some(val) = self.variables.get(&OsString::from(key)) {
                        print!("{}", val.to_string_lossy());
                        if peeker.peek().is_some() { print!(" "); }
                        continue;
                    } else {
                        print!("{}", arg);
                    }
                } else {
                    print!("{}", arg);
                }
            } else {
                break;
            }
            if peeker.peek().is_some() {
                print!(" ");
            }
        }
       println!();
    }
}