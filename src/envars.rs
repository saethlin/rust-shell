extern crate std;

use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::env;

#[derive(Default)]
pub struct Envars {
    map: HashMap<std::ffi::OsString, std::ffi::OsString>,
}

impl Envars {
    pub fn load() -> Self {
        let mut this = Envars { map: HashMap::new() };
        for (key, value) in env::vars_os() {
            this.map.insert(key, value);
        }
        return this;
    }

    pub fn get(&self, name: &str) -> Option<&std::ffi::OsString> {
        self.map.get(OsStr::new(name))
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.map.insert(OsString::from(key), OsString::from(value));
        env::set_var(key, value);
    }
}
