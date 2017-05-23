use std::collections::HashMap;
use std::path::PathBuf;

pub struct ShellState {
    pub directory: PathBuf,
    pub user: String,
    pub hostname: String,
    pub variables: HashMap<String, String>,
    pub input_buffer: String,
    pub output_buffer: String,
}
