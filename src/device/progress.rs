//! Each detective's arrests, by name, kept in flash under `progress` as JSON
//! (`{"jon": 3}`). Names are matched as Carmen's shared crate matches them:
//! trimmed and in lower case, so "Jon" and " jon " are one detective.

use serde_json::{Map, Value};

use crate::device::storage;

/// The longest name a detective can sign in with, in characters.
pub const MAX_NAME: usize = 8;

const STORE: &str = "progress";

pub struct Progress {
    arrests: Map<String, Value>,
}

impl Progress {
    pub fn load() -> Progress {
        let saved = storage::read(STORE).and_then(|text| serde_json::from_str::<Value>(&text).ok());
        Progress { arrests: saved.and_then(|v| v.as_object().cloned()).unwrap_or_default() }
    }

    pub fn arrests(&self, name: &str) -> u32 {
        key(name).and_then(|k| self.arrests.get(&k)?.as_u64()).unwrap_or(0) as u32
    }

    /// Counts an arrest for `name` and saves at once; returns the new total.
    pub fn add_arrest(&mut self, name: &str) -> u32 {
        let total = self.arrests(name) + 1;
        if let Some(k) = key(name) {
            self.arrests.insert(k, total.into());
            storage::write(STORE, &Value::Object(self.arrests.clone()).to_string());
        }
        total
    }
}

/// The name records are filed under; None if it's empty or too long.
fn key(name: &str) -> Option<String> {
    let name: String = name.chars().filter(|c| !c.is_control()).collect::<String>().trim().to_string();
    (1..=MAX_NAME).contains(&name.chars().count()).then(|| name.to_lowercase())
}
