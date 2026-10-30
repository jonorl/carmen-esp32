//! Postcards for the device's picture frame. The landmarks are drawn at fixed
//! positions for the game's 274x282 frame, so painting straight into a
//! smaller frame cuts them off; instead each postcard is painted at that size
//! and shrunk to the frame's width, averaging the pixels it folds together,
//! with the top of the sky carried up into any room left above.

use crate::art::pixels::{Canvas, Rgb};
use crate::art::scene::{self, TimeOfDay};

/// The size the landmarks are drawn for.
pub const DESIGNED: (i32, i32) = (274, 282);

/// The whole postcard, shrunk into `w` x `h`.
pub fn fitted(country: &str, lat: f64, time: TimeOfDay, w: i32, h: i32) -> Canvas {
    let full = scene::paint(country, lat, time, DESIGNED.0, DESIGNED.1);
    shrink(&full, w, h)
}

/// The postcard at full size, seen through a `w` x `h` window centred across
/// it and resting on its bottom edge: crisp, but the sides and the tops of
/// tall landmarks are lost.
pub fn windowed(country: &str, lat: f64, time: TimeOfDay, w: i32, h: i32) -> Canvas {
    let full = scene::paint(country, lat, time, DESIGNED.0, DESIGNED.1);
    let (x0, y0) = ((DESIGNED.0 - w) / 2, DESIGNED.1 - h);
    let mut out = Canvas::new(w, h);
    for y in 0..h {
        for x in 0..w {
            if let Some(c) = full.get(x0 + x, y0 + y) {
                out.set(x, y, c);
            }
        }
    }
    out
}

/// `src` scaled to `w` wide, keeping its shape, at the bottom of a `w` x `h`
/// canvas; rows above it repeat its top row.
fn shrink(src: &Canvas, w: i32, h: i32) -> Canvas {
    let k = src.width() as f32 / w as f32; // source pixels per output pixel
    let scaled_h = ((src.height() as f32 / k).round() as i32).min(h);
    let top = h - scaled_h;
    let mut out = Canvas::new(w, h);
    for y in 0..scaled_h {
        for x in 0..w {
            out.set(x, top + y, average(src, x as f32 * k, y as f32 * k, k));
        }
    }
    for y in 0..top {
        for x in 0..w {
            out.set(x, y, out.get(x, top).unwrap_or_default());
        }
    }
    out
}

/// The mean colour of the `k` x `k` square of `src` from (x, y), each pixel
/// weighted by how much of it the square covers.
fn average(src: &Canvas, x: f32, y: f32, k: f32) -> Rgb {
    let mut sum = [0.0f32; 3];
    let mut total = 0.0;
    let (x1, y1) = (x + k, y + k);
    for py in y.floor() as i32..y1.ceil() as i32 {
        let wy = (y1.min(py as f32 + 1.0) - y.max(py as f32)).max(0.0);
        for px in x.floor() as i32..x1.ceil() as i32 {
            let wx = (x1.min(px as f32 + 1.0) - x.max(px as f32)).max(0.0);
            if let Some(c) = src.get(px, py) {
                let wgt = wx * wy;
                for i in 0..3 {
                    sum[i] += c[i] as f32 * wgt;
                }
                total += wgt;
            }
        }
    }
    if total == 0.0 { [0, 0, 0] } else { sum.map(|v| (v / total).round() as u8) }
}
