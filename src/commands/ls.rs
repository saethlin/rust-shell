extern crate term_size;
use std::path::Path;
use std::fs;
use std::fs::File;
use super::option_parser::parse;
use state::ShellState;

pub fn exec(state: &ShellState, args: Vec<String>) {
    let (options, params) = parse(args);

    let dir: &Path = if !params.is_empty() {
        Path::new(&params[0])
    } else {
        &state.directory
    };

    let dir_path = Path::new(dir);
    let dir_entry = fs::read_dir(dir_path).unwrap().map(|x| x.unwrap());
    let details = options.contains(&"l".to_string());

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

        print_files(filtered, details);
    } else {
        print_files(dir_entry, details);
    };
}

fn print_files<T: Iterator<Item=fs::DirEntry>>(entries: T, detailed: bool) {
    use self::term_size::dimensions;
    let (width, height) = dimensions().unwrap();
    println!("{}, {}", width, height);

    // Try to divide into columns
    for num_cols in 1..width/3 {

    }

    for entry in entries {
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let file = File::open(&path).unwrap();
        let meta = file.metadata().unwrap();
        let size = meta.len();
        println!("{}\t{}", file_name, size);
    };
}
