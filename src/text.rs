//! Text from the game's fonts, drawn onto a `Canvas`. The fonts are turned
//! into bitmaps at build time (`build.rs`): Pixelify Sans and VT323 as their
//! own design pixels, fully on or off, and smoothed sets (Pixelify at 13 px
//! for body text, and Noto Sans for ₹). Each glyph keeps its coverage (0 to
//! 255) and is blended by it.

use crate::art::pixels::{Canvas, Rgb, mix};

include!(concat!(env!("OUT_DIR"), "/fonts.rs"));

pub struct Glyph {
    pub c: char,
    advance: f32,
    /// The bitmap's left edge from the pen, and its bottom edge above the baseline.
    x: i32,
    y: i32,
    w: usize,
    h: usize,
    /// Where its coverage starts in `FontData::alpha`, row by row.
    at: usize,
}

pub struct FontData {
    pub size: u32,
    ascent: i32,
    line: i32,
    /// Sorted by character.
    glyphs: &'static [Glyph],
    alpha: &'static [u8],
}

/// Which font a piece of text uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    /// Pixelify Sans as its own pixels: 11 px (one screen pixel each), for
    /// small labels, or 22 (each doubled), for headings.
    Pixelify,
    /// Pixelify Sans rasterised the ordinary way, smoothed, at 13 px: the
    /// body text, chosen on the screen over 11 and 14 px.
    PixelifySmooth,
    /// VT323, the printer's monospaced font: 25 px.
    Vt323,
}

#[derive(Clone, Copy)]
pub struct Font {
    data: &'static FontData,
    /// Noto Sans at the same size, for characters the font lacks (₹).
    fallback: Option<&'static FontData>,
}

impl Font {
    /// The font at `size` pixels, if it was built at that size (see `build.rs`).
    pub fn new(family: Family, size: u32) -> Option<Font> {
        let sets = match family {
            Family::Pixelify => PIXELIFY,
            Family::PixelifySmooth => PIXELIFY_SMOOTH,
            Family::Vt323 => VT323,
        };
        let data = *sets.iter().find(|d| d.size == size)?;
        let fallback = NOTO.iter().find(|d| d.size == size).copied();
        Some(Font { data, fallback })
    }

    pub fn size(&self) -> u32 {
        self.data.size
    }

    /// From one baseline to the next.
    pub fn line_height(&self) -> i32 {
        self.data.line
    }

    /// From the top of a line to its baseline.
    pub fn ascent(&self) -> i32 {
        self.data.ascent
    }

    fn glyph(&self, c: char) -> Option<(&'static FontData, &'static Glyph)> {
        let find = |d: &'static FontData| d.glyphs.binary_search_by_key(&c, |g| g.c).ok().map(|i| (d, &d.glyphs[i]));
        find(self.data).or_else(|| self.fallback.and_then(find))
    }

    /// Width in pixels; a character in neither font takes no room.
    pub fn width(&self, text: &str) -> i32 {
        text.chars().filter_map(|c| self.glyph(c)).map(|(_, g)| g.advance).sum::<f32>().round() as i32
    }

    /// Draws `text` with its baseline at `y`, starting at `x`; returns where the pen ends.
    pub fn draw(&self, cv: &mut Canvas, x: i32, y: i32, text: &str, colour: Rgb) -> i32 {
        let mut pen = x as f32;
        for c in text.chars() {
            let Some((data, g)) = self.glyph(c) else { continue };
            let (left, top) = (pen.round() as i32 + g.x, y - g.y - g.h as i32);
            for row in 0..g.h {
                for col in 0..g.w {
                    let a = data.alpha[g.at + row * g.w + col];
                    let (px, py) = (left + col as i32, top + row as i32);
                    if a == 255 {
                        cv.set(px, py, colour);
                    } else if a > 0
                        && let Some(under) = cv.get(px, py)
                    {
                        cv.set(px, py, mix(under, colour, a as f32 / 255.0));
                    }
                }
            }
            pen += g.advance;
        }
        pen.round() as i32
    }

    /// `text` broken into lines no wider than `width`, at spaces where it can;
    /// a word too long for a line is broken where it has to be.
    pub fn wrap(&self, text: &str, width: i32) -> Vec<String> {
        let mut lines = Vec::new();
        for paragraph in text.split('\n') {
            let mut line = String::new();
            for word in paragraph.split(' ') {
                let tried = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
                if self.width(&tried) <= width {
                    line = tried;
                    continue;
                }
                if !line.is_empty() {
                    lines.push(std::mem::take(&mut line));
                }
                for c in word.chars() {
                    if !line.is_empty() && self.width(&format!("{line}{c}")) > width {
                        lines.push(std::mem::take(&mut line));
                    }
                    line.push(c);
                }
            }
            lines.push(line);
        }
        lines
    }
}
