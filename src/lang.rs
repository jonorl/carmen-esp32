//! The game's two languages: English, and Spanish as spoken in Argentina
//! (with vos). One is in use at a time, set at start-up from what the player
//! chose before, or else from the system's or browser's language, and
//! changed with the toggle in the menu bar.
//!
//! Text is translated where it's written: `tr("Close", "Cerrar")` in the
//! code, and an "es" section beside the English in the data files. Anything
//! without a translation shows in English.

use std::cell::Cell;

use crate::device::storage;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    En,
    Es,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Es => "es",
        }
    }

    fn from_code(code: &str) -> Option<Lang> {
        let code = code.trim().trim_matches('"').to_lowercase();
        if code.starts_with("es") {
            Some(Lang::Es)
        } else if code.starts_with("en") {
            Some(Lang::En)
        } else {
            None
        }
    }
}

thread_local! {
    static NOW: Cell<Lang> = const { Cell::new(Lang::En) };
}

/// What it's saved under (see `storage`).
const STORE: &str = "language";

pub fn lang() -> Lang {
    NOW.get()
}

pub fn set(l: Lang) {
    NOW.set(l);
}

/// `en` or `es`, whichever language is in use.
pub fn tr<T>(en: T, es: T) -> T {
    match lang() {
        Lang::En => en,
        Lang::Es => es,
    }
}

/// Start in the language chosen last time, or else the system's.
pub fn start() {
    let saved = storage::read(STORE).and_then(|s| Lang::from_code(&s));
    set(saved.unwrap_or_else(|| if system::spanish() { Lang::Es } else { Lang::En }));
}

/// Switch to `l` and remember it for next time.
pub fn choose(l: Lang) {
    set(l);
    storage::write(STORE, &format!("\"{}\"", l.code()));
}

#[cfg(not(target_arch = "wasm32"))]
mod system {
    /// Whether the desktop is set to Spanish, by the usual locale variables
    /// (the first one set wins, as for any program).
    pub fn spanish() -> bool {
        ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"]
            .iter()
            .filter_map(|v| std::env::var(v).ok())
            .find(|v| !v.is_empty() && v != "C" && v != "POSIX")
            .is_some_and(|v| v.to_lowercase().starts_with("es"))
    }
}

#[cfg(target_arch = "wasm32")]
mod system {
    unsafe extern "C" {
        /// Copies the browser's preferred language ("es-AR", "en-GB"...) into
        /// `out` and returns its length.
        fn carmen_browser_language(out: *mut u8, cap: usize) -> i32;
    }

    pub fn spanish() -> bool {
        let mut buf = vec![0u8; 64];
        // SAFETY: the plugin writes at most `cap` bytes into `buf`.
        let len = unsafe { carmen_browser_language(buf.as_mut_ptr(), buf.len()) };
        buf.truncate(usize::try_from(len).unwrap_or(0));
        String::from_utf8(buf).is_ok_and(|code| code.to_lowercase().starts_with("es"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_language_codes() {
        assert_eq!(Lang::from_code("\"es\""), Some(Lang::Es));
        assert_eq!(Lang::from_code("es-AR"), Some(Lang::Es));
        assert_eq!(Lang::from_code("en"), Some(Lang::En));
        assert_eq!(Lang::from_code("fr"), None);
    }

    #[test]
    fn picks_the_language_in_use() {
        set(Lang::Es);
        assert_eq!(tr("Close", "Cerrar"), "Cerrar");
        set(Lang::En);
        assert_eq!(tr("Close", "Cerrar"), "Close");
    }
}
