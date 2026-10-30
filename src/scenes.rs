//! The animated moments in the text panel, each with its sounds: walking to
//! a witness, a henchman, the final city's warnings, a flight, the crime
//! computer working, the chase, the walk to jail and the jail. Timed as
//! Carmen's `screens/panel.rs` times them, from recordings of the original.

use crate::art::pixels::Canvas;
use crate::art::sprites::{self, MARCHING, RUNNING_POLICE, RUNNING_SUSPECT, SNEAKING, STRIDING, TRUDGING, popup};
use crate::device::sound::{self, Notes};
use crate::device::tunes;
use crate::lang::{Lang, lang, tr};
use crate::ui::{self, Fonts, Rect};

pub enum Stage {
    /// Walking to the place named, to footsteps.
    Walk(String),
    /// A henchman (0 striding, 1 sneaking, 2 popping up); `tell` the first
    /// time, when the player is told what he means.
    Henchman { kind: usize, tell: bool },
    /// A warning at a wrong place in the final city: 0 axe, 1 knife, 2 pistol.
    Danger(usize),
    /// Flying to the city named.
    Fly(String),
    /// The crime computer working.
    Compute,
    /// The thief spotted and chased; `female` for the Spanish line.
    Chase { female: bool },
    /// The walk to jail: the suspect at gunpoint, or the officer alone.
    March { caught: bool },
    /// In jail, or the empty cell.
    Jail { caught: bool },
}

// After the walk to a witness: the henchman's tune, and when each shows himself.
const HENCHMAN_TUNE_AT: f64 = 0.9;
const HENCHMAN_APPEARS: [f64; 3] = [HENCHMAN_TUNE_AT + 1.15, HENCHMAN_TUNE_AT + 1.14, HENCHMAN_TUNE_AT + 0.93];
const HENCHMAN_LINE_SECONDS: f64 = 3.5;
// The warnings: when the weapon shows, its tune starts, and the witness takes over.
const DANGER_AT: [f64; 3] = [1.05, 1.12, 0.88];
const DANGER_TUNE_AT: [f64; 3] = [1.0, 1.2, 1.69];
const DANGER_ENDS: [f64; 3] = [5.75, 5.05, 5.95];
// The chase, timed to its music.
const CHASE_SECONDS: f64 = 17.6;
const SUSPECT_AT: f64 = 1.6;
const POLICE_AT: f64 = 14.5;
// The walk to jail and the jail.
const MARCH_AT: f64 = 0.2;
const JAILED_AT: f64 = 9.8;
const TRUDGE_AT: f64 = 7.1;
const EMPTY_JAIL_AT: f64 = 15.1;
const CLANK_AT: f64 = 14.215;
const PRINT_AT: f64 = 14.75;
// The device's own: the desktop shows a walk on the place picker and a flight on the map.
const WALK_SECONDS: f64 = 1.86;
const FLY_SECONDS: f64 = 3.0;
const COMPUTE_SECONDS: f64 = 2.1;
const EMPTY_CELL_SECONDS: f64 = 2.5;

impl Stage {
    pub fn seconds(&self, panel: Rect) -> f64 {
        match self {
            Stage::Walk(_) => WALK_SECONDS,
            Stage::Henchman { kind, tell } => henchman_gone(*kind, panel) + if *tell { HENCHMAN_LINE_SECONDS } else { 0.0 },
            Stage::Danger(kind) => DANGER_ENDS[*kind],
            Stage::Fly(_) => FLY_SECONDS,
            Stage::Compute => COMPUTE_SECONDS,
            Stage::Chase { .. } => CHASE_SECONDS,
            Stage::March { caught: true } => JAILED_AT,
            Stage::March { caught: false } => EMPTY_JAIL_AT,
            Stage::Jail { caught: true } => PRINT_AT,
            Stage::Jail { caught: false } => EMPTY_CELL_SECONDS,
        }
    }

    /// What to play, and when from the stage's start.
    pub fn cues(&self) -> Vec<(f64, Notes)> {
        match self {
            Stage::Walk(_) => vec![(0.0, sound::WALK)],
            Stage::Henchman { kind, .. } => {
                vec![(HENCHMAN_TUNE_AT, [tunes::HENCHMAN_STRIDE, tunes::HENCHMAN_SNEAK, tunes::HENCHMAN_POPUP][*kind])]
            }
            Stage::Danger(kind) => vec![(DANGER_TUNE_AT[*kind], [tunes::DANGER_AXE, tunes::DANGER_KNIFE, tunes::DANGER_GUN][*kind])],
            // A short sweep: the original's flight tune was too much on the piezo.
            Stage::Fly(_) => vec![(0.0, sound::SHWOOM)],
            Stage::Compute => vec![(0.0, tunes::COMPUTING)],
            Stage::Chase { .. } => vec![(0.0, tunes::CHASE)],
            Stage::March { caught: true } => vec![(0.0, sound::FOOTSTEPS)],
            Stage::March { caught: false } => vec![(1.0, tunes::DISAPPOINT), (TRUDGE_AT, sound::FOOTSTEPS)],
            Stage::Jail { caught: true } => vec![(0.0, tunes::CAPTURED), (CLANK_AT, sound::CLANK)],
            Stage::Jail { caught: false } => vec![],
        }
    }

    /// Draws the stage `t` seconds in, in `panel`.
    pub fn draw(&self, cv: &mut Canvas, f: &Fonts, panel: Rect, t: f64) {
        match self {
            Stage::Walk(place) => {
                caption(cv, f, panel, &format!("{place}…"));
                // Footprints across the panel, one a step.
                let steps = ((t / WALK_SECONDS) * 8.0) as i32 + 1;
                for k in 0..steps.min(8) {
                    let x = panel.x + 12 + k * (panel.w - 24) / 8;
                    let y = panel.y + panel.h - 30 - if k % 2 == 0 { 0 } else { 6 };
                    cv.rect(x, y, 4, 6, ui::GREY);
                }
            }
            Stage::Henchman { kind, tell } => {
                let appears = t - HENCHMAN_APPEARS[*kind];
                match kind {
                    0 => STRIDING.draw(cv, panel, appears),
                    1 => SNEAKING.draw(cv, panel, appears),
                    _ => popup::draw(cv, panel, appears),
                }
                if *tell && t >= henchman_gone(*kind, panel) {
                    low_lines(
                        cv,
                        f,
                        panel,
                        tr(
                            ["A S.U.R.M.E. henchman!", "You must be on", "the right track."],
                            ["¡Un secuaz de S.U.R.M.E.!", "Vas por", "buen camino."],
                        ),
                    );
                }
            }
            Stage::Danger(kind) => sprites::danger(cv, *kind, panel, t - DANGER_AT[*kind]),
            Stage::Fly(city) => {
                caption(cv, f, panel, &tr(format!("Flying to {city}"), format!("Volando a {city}")));
                // A plane crossing the panel, the original's icon in miniature.
                let x = panel.x - 30 + ((panel.w + 60) as f64 * t / FLY_SECONDS) as i32;
                let y = panel.y + panel.h / 2;
                cv.rect(x, y, 28, 3, ui::WHITE);
                cv.rect(x + 12, y - 8, 4, 19, ui::WHITE);
                cv.rect(x, y - 4, 3, 11, ui::WHITE);
            }
            Stage::Compute => {
                caption(cv, f, panel, tr("Computing…", "Calculando…"));
                // The stand's lights, flashing the EGA colours at random.
                let colours = [ui::RED, ui::LIME, ui::YELLOW, ui::CYAN, ui::BLUE, ui::WHITE];
                let tick = (t * 12.0) as usize;
                for k in 0..6 {
                    let c = colours[(tick * 7 + k * 5 + tick / 3) % colours.len()];
                    cv.rect(panel.x + 20 + k as i32 * 21, panel.y + panel.h / 2, 14, 8, c);
                }
            }
            Stage::Chase { female } => {
                RUNNING_SUSPECT.draw(cv, panel, t - SUSPECT_AT);
                let (first, second) = match (lang(), female) {
                    (Lang::En, _) => ("There goes", "the suspect!"),
                    (Lang::Es, false) => ("¡Ahí va", "el sospechoso!"),
                    (Lang::Es, true) => ("¡Ahí va", "la sospechosa!"),
                };
                // As in the original, which wiped the lines one at a time.
                let first = if (5.7..9.8).contains(&t) { first } else { "" };
                let second = if (5.7..10.6).contains(&t) { second } else { "" };
                low_lines(cv, f, panel, [first, second, ""]);
                RUNNING_POLICE.draw(cv, panel, t - POLICE_AT);
            }
            Stage::March { caught: true } => MARCHING.draw(cv, panel, t - MARCH_AT),
            Stage::March { caught: false } => TRUDGING.draw(cv, panel, t - TRUDGE_AT),
            Stage::Jail { caught: true } => sprites::jail(cv, panel, Some(t >= CLANK_AT)),
            Stage::Jail { caught: false } => sprites::jail(cv, panel, None),
        }
    }
}

/// When henchman `kind` has gone from the panel.
fn henchman_gone(kind: usize, panel: Rect) -> f64 {
    HENCHMAN_APPEARS[kind]
        + match kind {
            0 => STRIDING.seconds(panel),
            1 => SNEAKING.seconds(panel),
            _ => popup::SECONDS,
        }
}

/// A line at the top of the panel.
fn caption(cv: &mut Canvas, f: &Fonts, panel: Rect, text: &str) {
    let mut y = panel.y + 6;
    for line in f.body.wrap(text, panel.w - 8) {
        f.body.draw(cv, panel.x + 4, y + f.body.ascent(), &line, ui::WHITE);
        y += f.body.line_height();
    }
}

/// Lines centred low in the panel, as the original showed its exclamations.
fn low_lines(cv: &mut Canvas, f: &Fonts, panel: Rect, lines: [&str; 3]) {
    let top = panel.y + panel.h * 2 / 5;
    for (n, line) in lines.iter().enumerate() {
        let x = panel.x + (panel.w - f.body.width(line)) / 2;
        f.body.draw(cv, x, top + n as i32 * f.body.line_height() + f.body.ascent(), line, ui::WHITE);
    }
}
