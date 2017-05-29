extern crate std;

use std::collections::HashMap;
use std::ffi::OsStr;

pub struct EnVars {
    map: HashMap<std::ffi::OsString, std::ffi::OsString>
}

impl EnVars {
    pub fn new() -> Self {
        EnVars {
            map: HashMap::new()
        }
    }

    pub fn get(&self, name: &str) -> Option<&std::ffi::OsString> {
        return self.map.get(OsStr::new(name));
    }
}