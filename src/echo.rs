extern crate std;
extern crate itertools;

use std::iter;
use self::itertools::Itertools;
use std::ffi::OsStr;
use state::ShellState;

impl ShellState {
    pub fn echo(&self, args: std::str::SplitWhitespace) {
        let vars = args.map(|a| self.lookup_envar(a).unwrap_or(a.to_owned()));
        for entry in iter::repeat(" ".to_owned()).interleave_shortest(vars).skip(1) {
            print!("{}", entry)
        };
        println!();
    }

    fn lookup_envar(&self, arg: &str) -> Option<String> {
        if arg.starts_with('$') {
            let (_, key) = arg.split_at(1);
            if let Some(val) = self.variables.get(OsStr::new(key)) {
                return Some(val.to_string_lossy().into_owned());
            }
        }
        None
    }
}
