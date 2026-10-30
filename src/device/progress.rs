//! Each detective's arrests, by name, kept in flash under `progress` as JSON
//! (`{"jon": 3}`) and on the bureau's server, so a rank follows its detective
//! between this device, the desktop game and the web, as Carmen's own
//! `device/progress.rs` does. The game never waits on the server for long:
//! without it, the copy in flash carries on alone, and the two catch up next
//! time. A tally only ever goes up, so catching up means taking the higher.
//! Names are matched as Carmen's shared crate matches them: trimmed and in
//! lower case, so "Jon" and " jon " are one detective.

use serde_json::{Map, Value, json};

use crate::device::net::{self, Net};
use crate::device::storage;

/// The longest name a detective can sign in with, in characters.
pub const MAX_NAME: usize = 8;

const STORE: &str = "progress";

/// A detective on the roster.
#[derive(Clone, Debug)]
pub struct Detective {
    pub name: String,
    pub arrests: u32,
}

/// What came back from the server, for the game to act on.
pub enum News {
    /// The server's word on a detective signing in (or that it had none).
    LookedUp,
    /// Everyone on file; None if the server couldn't be asked.
    Roster(Option<Vec<Detective>>),
}

enum Asked {
    /// Their arrests, as they sign in.
    LookUp(String),
    /// Their tally sent to be kept.
    Tally,
    Roster,
}

pub struct Progress {
    arrests: Map<String, Value>,
    net: Option<Net>,
    pending: Vec<(u32, Asked)>,
}

impl Progress {
    pub fn load(net: Option<Net>) -> Progress {
        let saved = storage::read(STORE).and_then(|text| serde_json::from_str::<Value>(&text).ok());
        Progress { arrests: saved.and_then(|v| v.as_object().cloned()).unwrap_or_default(), net, pending: Vec::new() }
    }

    pub fn arrests(&self, name: &str) -> u32 {
        key(name).and_then(|k| self.arrests.get(&k)?.as_u64()).unwrap_or(0) as u32
    }

    /// Counts an arrest for `name`, saves at once, and tells the server; returns the new total.
    pub fn add_arrest(&mut self, name: &str) -> u32 {
        let total = self.arrests(name) + 1;
        self.set(name, total);
        self.send_tally(name);
        total
    }

    fn set(&mut self, name: &str, arrests: u32) {
        if let Some(k) = key(name) {
            self.arrests.insert(k, arrests.into());
            storage::write(STORE, &Value::Object(self.arrests.clone()).to_string());
        }
    }

    /// Asks the server for `name`'s arrests as they sign in; false if there's
    /// no server to ask (then don't wait for `News::LookedUp`).
    pub fn look_up(&mut self, name: &str) -> bool {
        let (Some(url), Some(net)) = (detective_url(name), self.net.as_mut()) else { return false };
        let id = net.get(&url);
        self.pending.push((id, Asked::LookUp(name.to_string())));
        true
    }

    fn send_tally(&mut self, name: &str) {
        let arrests = self.arrests(name);
        let (Some(url), Some(net)) = (detective_url(name), self.net.as_mut()) else { return };
        let id = net.put(&url, json!({ "arrests": arrests }).to_string());
        self.pending.push((id, Asked::Tally));
    }

    /// Asks the server for everyone; false if there's no server to ask.
    pub fn ask_roster(&mut self) -> bool {
        let (Some(server), Some(net)) = (net::server(), self.net.as_mut()) else { return false };
        let id = net.get(&format!("{server}/detectives"));
        self.pending.push((id, Asked::Roster));
        true
    }

    /// Everyone on this device, most arrests first: the roster without the server.
    pub fn local_roster(&self) -> Vec<Detective> {
        let mut list: Vec<Detective> =
            self.arrests.iter().map(|(name, n)| Detective { name: name.clone(), arrests: n.as_u64().unwrap_or(0) as u32 }).collect();
        list.sort_by(|a, b| b.arrests.cmp(&a.arrests).then_with(|| a.name.cmp(&b.name)));
        list
    }

    /// Takes in whatever the server has answered. Call often.
    pub fn poll(&mut self) -> Vec<News> {
        let Some(net) = &self.net else { return Vec::new() };
        let mut news = Vec::new();
        for (id, answer) in net.poll() {
            let Some(at) = self.pending.iter().position(|(p, _)| *p == id) else { continue };
            let (_, asked) = self.pending.remove(at);
            match asked {
                Asked::LookUp(name) => {
                    // The server answers 0 for a name it doesn't know; it never lowers a tally.
                    if let Some(theirs) = answer.ok().and_then(|body| arrests_in(&body))
                        && theirs > self.arrests(&name)
                    {
                        self.set(&name, theirs);
                    }
                    // Always sent back: it puts a new detective on file (and on the
                    // roster), and brings in arrests made here while out of reach.
                    self.send_tally(&name);
                    news.push(News::LookedUp);
                }
                Asked::Tally => {}
                Asked::Roster => {
                    let list = answer.ok().and_then(|body| serde_json::from_str::<Value>(&body).ok()).and_then(|v| {
                        v.as_array().map(|all| {
                            all.iter()
                                .filter_map(|d| {
                                    Some(Detective { name: d["name"].as_str()?.to_string(), arrests: d["arrests"].as_u64()? as u32 })
                                })
                                .collect()
                        })
                    });
                    news.push(News::Roster(list));
                }
            }
        }
        news
    }
}

fn arrests_in(body: &str) -> Option<u32> {
    serde_json::from_str::<Value>(body).ok()?["arrests"].as_u64().map(|n| n as u32)
}

fn detective_url(name: &str) -> Option<String> {
    key(name)?;
    Some(format!("{}/detectives/{}", net::server()?, net::escape(name.trim())))
}

/// The name records are filed under; None if it's empty or too long.
pub fn key(name: &str) -> Option<String> {
    let name: String = name.chars().filter(|c| !c.is_control()).collect::<String>().trim().to_string();
    (1..=MAX_NAME).contains(&name.chars().count()).then(|| name.to_lowercase())
}
