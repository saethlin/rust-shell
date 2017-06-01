extern crate std;

use std::io::{BufReader, BufRead};
use state::ShellState;
use std::path::Path;

impl ShellState {
    pub fn read_config(&mut self) {
        let config_path = Path::new(self.variables.get("HOME").unwrap()).join(Path::new(".rshrc"));
        if let Ok(f) =  std::fs::File::open(config_path) {
            let reader = BufReader::new(f);
            for line in reader.lines().filter_map(|l| l.ok()).filter(|l| l.contains('=')) {
                let mut split = line.as_str().split('=');
                let key = split.next().unwrap();
                let value = split.next().unwrap_or("").to_owned();
                let real_value = value.replace("\\n", "\n");
                self.variables.insert(key, real_value.as_ref());
            }
        }
    }
}