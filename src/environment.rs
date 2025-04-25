use std::ffi::{OsStr, OsString};

use crate::Shell;

pub struct EnvCollector {
    shell: Shell,
    buffer: Option<OsString>,
}

impl EnvCollector {
    pub fn new(shell: Shell) -> Self {
        Self {
            shell,
            buffer: None,
        }
    }

    pub fn acc(&mut self, key: &OsStr, _val: &OsStr) {
        match self.shell {
            Shell::Wsl => {
                let buffer = self
                    .buffer
                    .get_or_insert_with(|| OsString::with_capacity(key.len() + 1));
                buffer.push(key);
                buffer.push(":");
            }
            _ => {}
        }
    }

    pub fn collect(self) -> Option<(OsString, OsString)> {
        match self.shell {
            Shell::Wsl => self.buffer.map(|val| (OsString::from("WSLENV"), val)),
            _ => None,
        }
    }
}
