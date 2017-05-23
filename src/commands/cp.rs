use std::path::Path;
use std::fs;
use state::ShellState;

pub fn exec(state: &mut ShellState, args: &[String]) {
