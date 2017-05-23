extern crate std;
extern crate term_size;
use std::path::Path;
use std::fs;
use super::option_parser::parse;
use state::ShellState;

pub fn exec(state: &ShellState, args: std::str::SplitWhitespace) {
    let (options, params) = parse(args);

    let dir: &Path = if !params.is_empty() {
        Path::new(&params[0])
    } else {
        &state.directory
    };

    let dir_path = Path::new(dir);
    let dir_entry = fs::read_dir(dir_path).unwrap().map(|x| x.unwrap());
    //let details = options.contains(&"l".to_string());

    if !&options.contains(&"a".to_string()) {
        let filtered = dir_entry.filter_map(|entry| {
            let path = entry.path();
            let is_hidden = path.file_name().unwrap().to_str().unwrap().starts_with('.');

            if is_hidden {
                None
            } else {
                Some(entry)
            }
        });

        print_files(filtered);
    } else {
        print_files(dir_entry);
    };
}

fn print_files<T: Iterator<Item=fs::DirEntry>>(entries: T) {
    use self::term_size::dimensions;
    let (terminal_width, terminal_height) = dimensions().unwrap();

    let mut sorted = Vec::new();
    for entry in entries {
        sorted.push(entry.path().file_name().unwrap().to_str().unwrap().to_owned());
    }
    sorted.sort_by_key(|e| e.to_lowercase());

    if sorted.is_empty() {return;}

    let mut cols = 0;
    // Try to divide into columns
    // The max number of columns would be a bunch of single-char entries, with 2 space padding
    for num_cols in 1..terminal_width/3 {
        let col_height = (sorted.len() / num_cols) + 1;
        let mut total_width = 0;
        for col_num in 0..num_cols {
            total_width += sorted
                .iter()
                .skip(col_num*col_height)
                .take(col_height)
                .map(|s| s.len()+2)
                .max().unwrap_or(0);
        }
        //println!("With for {} columns is {}", num_cols, total_width);
        if total_width > terminal_width {
            cols = num_cols-1;
            break;
        }
    }

    cols = 5;
    //println!("{}", cols);
    //println!("{}", (sorted.len() / cols) + 1);
    let rows = (sorted.len() / cols) + 1;

    let mut widths = Vec::with_capacity(cols);
    for c in 0..cols {
        let width = sorted.iter().map(|e| e.len()).skip(rows*c).take(rows).max().unwrap_or(0);
        widths.push(width);
    }

    for row_num in 0..rows {
        for (e, entry) in sorted.iter().skip(row_num).step_by(rows).take(cols).enumerate() {
            print!("{}", entry);
            for _ in 0..(widths[e] - entry.len() + 2) {
                print!(" ")
            }
        }
        println!();
    }
}
