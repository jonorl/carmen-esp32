//! The original game's little characters: the chase, the walk to jail, the
//! S.U.R.M.E. henchmen, the final city's warnings and the jail, traced from
//! recordings of it (in Carmen's `assets/sprites`, copied here). The timing
//! follows Carmen's `art/sprites.rs` and `art/jail.rs`; the drawing is the
//! device's own, onto a `Canvas`. The desktop doubles them for its bigger
//! panel; this one is about the original's size, so they're drawn at their
//! own: one screen pixel each.

use crate::art::pixels::Canvas;
use crate::ui::Rect;

/// A strip of same-size frames side by side, as RGBA (`build.rs`).
pub struct Strip {
    pub w: u32,
    pub h: u32,
    rgba: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/sprites.rs"));

impl Strip {
    /// Draws columns `from..to` and the top `rows` rows of frame `frame`
    /// (`fw` wide) with that part's top left at (x, y), clipped to `clip`'s
    /// sides. Black and see-through pixels are left out.
    fn draw(&self, cv: &mut Canvas, fw: i32, frame: usize, x: i32, y: i32, rows: i32, clip: Rect) {
        let x0 = frame as i32 * fw;
        for row in 0..rows.min(self.h as i32) {
            for col in 0..fw {
                let sx = x + col;
                if sx < clip.x || sx >= clip.x + clip.w {
                    continue;
                }
                let i = (((row as u32) * self.w + (x0 + col) as u32) * 4) as usize;
                let [r, g, b, a] = [self.rgba[i], self.rgba[i + 1], self.rgba[i + 2], self.rgba[i + 3]];
                if a >= 128 && (r, g, b) != (0, 0, 0) {
                    cv.set(sx, y + row, [r, g, b]);
                }
            }
        }
    }
}

/// One character walking or running across the bottom of a panel.
pub struct Anim {
    strip: &'static Strip,
    /// One frame's size.
    w: i32,
    h: i32,
    /// Ticks a second, and pixels moved each tick (negative is leftwards).
    rate: f64,
    step: i32,
    /// The frame shown on each tick of the cycle.
    cycle: &'static [usize],
}

impl Anim {
    /// How long it takes to cross `panel` and be gone.
    pub fn seconds(&self, panel: Rect) -> f64 {
        ((panel.w + self.w) as f64 / self.step.abs() as f64).ceil() / self.rate
    }

    /// Draws it `t` seconds after it set off from just outside `panel`,
    /// standing on the panel's bottom edge and clipped to its sides.
    pub fn draw(&self, cv: &mut Canvas, panel: Rect, t: f64) {
        if t < 0.0 {
            return;
        }
        let tick = (t * self.rate) as usize;
        // The first tick already shows one step of it.
        let moved = self.step * (tick + 1) as i32;
        let x = if self.step > 0 { panel.x - self.w + moved } else { panel.x + panel.w + moved };
        let frame = self.cycle[tick % self.cycle.len()];
        self.strip.draw(cv, self.w, frame, x, panel.y + panel.h - self.h, self.h, panel);
    }
}

const fn anim(strip: &'static Strip, w: i32, h: i32, rate: f64, step: i32, cycle: &'static [usize]) -> Anim {
    Anim { strip, w, h, rate, step, cycle }
}

/// Running off to the right: eight frames, 4 pixels a tick, 15 ticks a second.
pub static RUNNING_SUSPECT: Anim = anim(&SUSPECT, 47, 50, 15.0, 4, &[0, 1, 2, 3, 4, 5, 6, 7]);
/// Three officers giving chase: five frames, 4 pixels a tick, 20 a second.
pub static RUNNING_POLICE: Anim = anim(&POLICE, 49, 49, 20.0, 4, &[0, 1, 2, 3, 4]);
/// The suspect, hands up, walked off to the left at gunpoint.
pub static MARCHING: Anim = anim(&MARCH, 49, 49, 12.0, -2, &[0, 1, 2, 3, 4, 5]);
/// The officer trudging off alone when the suspect got away.
pub static TRUDGING: Anim = anim(&SLUMPED, 39, 45, 15.0, -2, &[0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9]);
/// A henchman striding across to the right, waving.
pub static STRIDING: Anim = anim(&HENCHMAN_STRIDE, 46, 42, 15.0, 2, &[0, 1, 2, 3, 4, 5, 6, 7]);
/// A henchman sneaking past to the left with a sack.
pub static SNEAKING: Anim = anim(&HENCHMAN_SNEAK, 45, 41, 20.0, -2, &[0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7]);

/// The henchman who pops up from the bottom of the panel: he rises a pixel
/// every thirtieth of a second, looks about, and sinks back down. Frames:
/// facing front, looking right, looking left, grinning.
pub mod popup {
    use super::*;

    const W: i32 = 42;
    const H: i32 = 32;
    const RISE: f64 = H as f64 / 30.0;
    const HOLD: f64 = 2.53;
    /// While he's up: from when, which frame.
    const LOOKS: [(f64, usize); 6] = [(0.0, 0), (0.5, 1), (1.0, 0), (1.5, 2), (2.0, 3), (2.25, 0)];
    pub const SECONDS: f64 = RISE + HOLD + RISE;

    /// Draws him `t` seconds after he starts rising, in the middle of the panel's bottom edge.
    pub fn draw(cv: &mut Canvas, panel: Rect, t: f64) {
        let (rows, frame) = if !(0.0..SECONDS).contains(&t) {
            return;
        } else if t < RISE {
            ((t * 30.0) as i32, 0)
        } else if t < RISE + HOLD {
            let up = t - RISE;
            (H, LOOKS.iter().rev().find(|(at, _)| up >= *at).map_or(0, |&(_, f)| f))
        } else {
            (H - ((t - RISE - HOLD) * 30.0) as i32, 0)
        };
        if rows >= 1 {
            HENCHMAN_POPUP.draw(cv, W, frame, panel.x + (panel.w - W) / 2, panel.y + panel.h - rows, rows, panel);
        }
    }
}

/// The final city's warnings: an axe spinning in from the left to bury
/// itself in the right-hand edge, a knife thrown in from the right into the
/// left-hand edge, and a hand with a pistol sliding in from the left,
/// cocking it and firing a shot across. Each stays once it has landed or fired.
pub fn danger(cv: &mut Canvas, kind: usize, panel: Rect, t: f64) {
    if t < 0.0 {
        return;
    }
    let bottom = panel.y + panel.h;
    match kind {
        0 => {
            // A quarter turn every 4 ticks, 6 pixels a tick at 30 a second.
            let tick = (t * 30.0) as usize;
            let x = panel.x - 44 + 6 * (tick + 1) as i32;
            let stuck = panel.x + panel.w - 44;
            let (x, frame) = if x >= stuck { (stuck, 4) } else { (x, tick / 4 % 4) };
            DANGER_AXE.draw(cv, 44, frame, x, bottom - 7 - 43, 43, panel);
        }
        1 => {
            // Six pixels every sixtieth of a second, until its tip is 5 pixels into the left-hand edge.
            let moved = 6 * ((t * 60.0) as i32 + 1);
            let x = (panel.x + panel.w - moved).max(panel.x - 5);
            DANGER_KNIFE.draw(cv, 59, 0, x, bottom - 27 - 14, 14, panel);
        }
        _ => {
            // In at 2 pixels a sixtieth of a second, pulled back, raised and back again, then the shot.
            let slid = (-49 + 2 * (t * 60.0) as i32).min(1);
            let frame = match t {
                t if (0.533..0.633).contains(&t) || (0.733..0.833).contains(&t) => 1,
                t if (0.633..0.733).contains(&t) => 2,
                _ => 0,
            };
            DANGER_GUN.draw(cv, 51, frame, panel.x + slid, bottom - 14 - 27, 27, panel);
            let bullet = panel.x + 50 + (240.0 * (t - 0.933)) as i32;
            if t >= 0.933 && bullet < panel.x + panel.w {
                cv.rect(bullet, bottom - 34, 6.min(panel.x + panel.w - bullet), 1, [239, 243, 243]);
            }
        }
    }
}

/// The jail, centred at the top of `panel`: the prisoner, gripping the bars
/// once `Some(true)`, or an empty cell for None.
pub fn jail(cv: &mut Canvas, panel: Rect, prisoner: Option<bool>) {
    let frame = match prisoner {
        Some(false) => 0,
        Some(true) => 1,
        None => 2,
    };
    let (w, h) = (126, 121);
    JAIL.draw(cv, w, frame, panel.x + (panel.w - w) / 2, panel.y + 4, h, panel);
}
