extern crate std;

use std::path::Path;
use std::fs;
use state::ShellState;

pub fn exec(state: &mut ShellState, args: std::str::SplitWhitespace) {
    if args.is_empty() {
        state.directory = Path::new(&state.variables["HOME"]).to_owned();
        return;
    }

    let dir = &args[0];
    // Match . and .. and return early if needed
    match dir.as_ref() {
        "." => return,
        ".." => state.directory = state.directory.parent().unwrap().to_owned(),
        _ => {
            let new_path = state.directory.join(Path::new(&dir));
            if check_folder(&new_path) {
                state.directory = new_path;
            } else {
                println!("Directory not found {}", new_path.to_string_lossy());
            }
        }
    }
}

fn check_folder(directory: &Path) -> bool {
    match fs::read_dir(directory) {
        Ok(..) => true,
        Err(..) => false
    }
}
