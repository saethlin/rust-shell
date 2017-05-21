use super::option_parser::parse;
use state::ShellState;

pub fn exec(state: &ShellState, args: Vec<String>) {
    let (_, params) = parse(args);

    if params.is_empty() {
        println!();
    }
    else {
        if params[0].starts_with('$') {
            let (_, key) = params[0].split_at(1);
            if let Some(val) = state.variables.get(key) {
                print!("{}", val);
            }
        }
        else {
            print!("{}", params[0]);
        }

        for param in params.iter().skip(1) {
            if param.starts_with('$') {
                let (_, key) = param.split_at(1);
                if let Some(val) = state.variables.get(key) {
                    print!(" {}", val);
                }
            }
            else {
                print!(" {}", param);
            }
        }
        println!();
    }
}
