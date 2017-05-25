extern crate std;

use state::ShellState;

pub fn exec(state: &ShellState, args: &mut std::str::SplitWhitespace) {

    let mut peeker = args.peekable();
    loop {
        if let Some(arg) = peeker.next() {
            if arg.starts_with('$') {
                let (_, key) = arg.split_at(1);
                if let Some(val) = state.variables.get(key) {
                    print!("{}", val);
                    continue;
                }
                else {
                    print!("{}", arg);
                }
            }
            else {
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
