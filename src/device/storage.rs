//! Small things kept between games, each under a name, in the flash's NVS
//! partition: the same `read` and `write` as the game's own storage, which
//! keeps them in files or the browser. Until `start` has run, reads find
//! nothing and writes are dropped.

use std::sync::{Mutex, OnceLock};

use esp_idf_svc::nvs::{EspDefaultNvs, EspDefaultNvsPartition};
use log::warn;

/// NVS keys and namespaces are at most 15 characters.
const NAMESPACE: &str = "carmen";

static NVS: OnceLock<Mutex<EspDefaultNvs>> = OnceLock::new();

pub fn start(partition: EspDefaultNvsPartition) -> anyhow::Result<()> {
    let nvs = EspDefaultNvs::new(partition, NAMESPACE, true)?;
    let _ = NVS.set(Mutex::new(nvs));
    Ok(())
}

pub fn read(name: &str) -> Option<String> {
    let nvs = NVS.get()?.lock().ok()?;
    let len = nvs.str_len(name).ok()??;
    let mut buf = vec![0; len];
    nvs.get_str(name, &mut buf).ok()?.map(str::to_string)
}

pub fn write(name: &str, text: &str) {
    let Some(nvs) = NVS.get() else { return };
    let saved = nvs.lock().map_err(|e| e.to_string()).and_then(|nvs| nvs.set_str(name, text).map_err(|e| e.to_string()));
    if let Err(e) = saved {
        warn!("couldn't save {name}: {e}");
    }
}
