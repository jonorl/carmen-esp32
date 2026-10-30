//! Links ESP-IDF, and turns the fonts in `assets/fonts` into bitmaps
//! (`$OUT_DIR/fonts.rs`, included by `src/text.rs`), so the device only
//! copies pixels.
//!
//! Pixelify Sans and VT323 are pixel fonts: their outlines are drawn on a
//! grid of square cells, one per design pixel. Rasterising them at an
//! ordinary size smears every cell over two screen pixels, since the grid
//! doesn't start at zero, so each glyph is rasterised large instead and every
//! cell lit if enough of it is covered. That gives the font's own pixels, one
//! screen pixel each (or a square of them at a larger scale). Noto Sans,
//! there only for ₹, is rasterised the ordinary way.

use std::collections::HashMap;
use std::fmt::Write;

struct PixelFont {
    name: &'static str,
    file: &'static str,
    /// The grid, in font units: a cell's size, and where cell (0, 0) starts.
    cell: f32,
    x0: f32,
    y0: f32,
    /// How much of a cell must be covered to light it. Pixelify's strokes
    /// spill about 10 units into the next cell and its accents sit half a
    /// cell off the grid, so a half keeps both to one pixel; VT323 draws
    /// scanlines inside its cells, so needs less.
    lit: f32,
    /// Whether some marks are drawn finer than the grid, as Pixelify's
    /// accents are; see `cells_lit`.
    fine_marks: bool,
    /// Sizes in pixels (one em). At the font's own size each design pixel is
    /// one screen pixel; at other sizes design pixels are scaled to whole
    /// screen pixels, some a pixel bigger than others.
    sizes: &'static [u32],
    /// Sizes to also rasterise the ordinary way, smoothed, as `<name>_SMOOTH`.
    smooth: &'static [u32],
}

/// Measured from the outlines (with fontTools): Pixelify's 'I' runs from 60
/// to 343 across and from -12 to 631 up, three cells by seven of 91 units;
/// VT323's cells are 40 units from zero.
const PIXEL_FONTS: [PixelFont; 2] = [
    PixelFont {
        name: "PIXELIFY",
        file: "PixelifySans.ttf",
        cell: 91.0,
        x0: 60.0,
        y0: -12.0,
        lit: 0.5,
        fine_marks: true,
        // 11 for labels and 22 for headings, as its own pixels; body text
        // is 13 smoothed, chosen on the screen over 11 and 14.
        sizes: &[11, 22],
        smooth: &[13],
    },
    PixelFont { name: "VT323", file: "VT323.ttf", cell: 40.0, x0: 0.0, y0: 0.0, lit: 0.3, fine_marks: false, sizes: &[25], smooth: &[] },
];

/// Big-raster pixels along a cell's side when sampling coverage.
const SAMPLES: f32 = 12.0;
/// How much of a cell a fine mark must cover to light it (see `cells_lit`).
const MARK_LIT: f32 = 0.2;

fn main() {
    embuild::espidf::sysenv::output();
    println!("cargo:rerun-if-changed=assets/fonts");
    println!("cargo:rerun-if-changed=build.rs");

    let mut out = String::new();
    let mut sizes = Vec::new();
    for pf in &PIXEL_FONTS {
        let font = load(pf.file);
        let units = font.units_per_em();
        let big = SAMPLES * units / pf.cell; // px per em when rasterised large
        let cells = |u: f32| u / pf.cell;
        let line = font.horizontal_line_metrics(units).expect("horizontal metrics");
        let (ascent, descent) = (cells(line.ascent - pf.y0).round() as i32, cells(-line.descent + pf.y0).round() as i32);
        let own = cells(units).round(); // the font's own size, in pixels
        let mut sets = Vec::new();
        for &size in pf.sizes {
            let k = size as f32 / own; // screen pixels per design pixel
            // Where a design pixel's edge falls on the screen, so neighbouring cells meet exactly.
            let edge = |n: i32| (n as f32 * k).floor() as i32;
            let (mut glyphs, mut alpha) = (String::new(), Vec::new());
            for c in chars(&font) {
                let (m, raster) = font.rasterize(c, big);
                let ink: Vec<(f32, f32, f32)> = (0..m.height)
                    .flat_map(|row| (0..m.width).map(move |col| (row, col)))
                    .filter_map(|(row, col)| {
                        let a = raster[row * m.width + col] as f32 / 255.0;
                        let ux = (m.xmin as f32 + col as f32 + 0.5) * units / big;
                        let uy = (m.ymin as f32 + (m.height - 1 - row) as f32 + 0.5) * units / big;
                        (a > 0.0).then_some((ux, uy, a))
                    })
                    .collect();
                let lit = cells_lit(pf, &ink);
                let advance = edge(cells(font.metrics(c, units).advance_width).round() as i32) as f32;
                let at = alpha.len();
                let (x, y, w, h) = if lit.is_empty() {
                    (0, 0, 0, 0)
                } else {
                    let (i0, i1) = (lit.iter().map(|k| k.0).min().unwrap(), lit.iter().map(|k| k.0).max().unwrap());
                    let (j0, j1) = (lit.iter().map(|k| k.1).min().unwrap(), lit.iter().map(|k| k.1).max().unwrap());
                    let (left, bottom, top) = (edge(i0), edge(j0), edge(j1 + 1));
                    let (w, h) = (edge(i1 + 1) - left, top - bottom);
                    let mut bitmap = vec![0u8; (w * h) as usize];
                    for &(i, j) in &lit {
                        for sy in edge(j)..edge(j + 1) {
                            for sx in edge(i)..edge(i + 1) {
                                bitmap[((top - 1 - sy) * w + sx - left) as usize] = 255;
                            }
                        }
                    }
                    alpha.extend(bitmap);
                    (left, bottom, w, h)
                };
                writeln!(glyphs, "        Glyph {{ c: {c:?}, advance: {advance:?}, x: {x}, y: {y}, w: {w}, h: {h}, at: {at} }},").unwrap();
            }
            sizes.push(size);
            sets.push(font_data(&mut out, &format!("{}_{size}", pf.name), size, edge(ascent), edge(ascent + descent), &glyphs, &alpha));
        }
        writeln!(out, "pub static {}: &[&FontData] = &[{}];", pf.name, sets.join(", ")).unwrap();
        let smooth: Vec<String> = pf.smooth.iter().map(|&size| smooth_set(&mut out, &font, &format!("{}_SMOOTH", pf.name), size)).collect();
        writeln!(out, "pub static {}_SMOOTH: &[&FontData] = &[{}];", pf.name, smooth.join(", ")).unwrap();
        sizes.extend(pf.smooth);
    }

    // Noto Sans for ₹, smoothed, at every size the pixel fonts come in.
    let noto = load("NotoSans-Regular.ttf");
    sizes.sort();
    sizes.dedup();
    let sets: Vec<String> = sizes.iter().map(|&size| smooth_set(&mut out, &noto, "NOTO", size)).collect();
    writeln!(out, "pub static NOTO: &[&FontData] = &[{}];", sets.join(", ")).unwrap();

    let path = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("fonts.rs");
    std::fs::write(path, out).unwrap();
}

/// The cells covered at least `lit`, from the big raster's ink (font-unit
/// position and coverage of each of its pixels). With `fine_marks`, ink left
/// outside those cells lights the cells it covers at least `MARK_LIT`:
/// Pixelify draws its acute and grave accents in half-cell steps, so no cell
/// is half covered by one, and they'd vanish.
fn cells_lit(pf: &PixelFont, ink: &[(f32, f32, f32)]) -> Vec<(i32, i32)> {
    let cell_of = |ux: f32, uy: f32| (((ux - pf.x0) / pf.cell).floor() as i32, ((uy - pf.y0) / pf.cell).floor() as i32);
    let area = SAMPLES * SAMPLES;
    let mut cover: HashMap<(i32, i32), f32> = HashMap::new();
    for &(ux, uy, a) in ink {
        *cover.entry(cell_of(ux, uy)).or_default() += a;
    }
    let mut lit: Vec<(i32, i32)> = cover.iter().filter(|&(_, &sum)| sum >= pf.lit * area).map(|(&k, _)| k).collect();
    if pf.fine_marks {
        // Only ink away from the lit cells: a stroke's spill into its
        // neighbours (about 10 units) mustn't count.
        let near = |(i, j): (i32, i32)| lit.iter().any(|&(li, lj)| (li - i).abs() <= 1 && (lj - j).abs() <= 1);
        let marks: Vec<(i32, i32)> = cover.iter().filter(|&(&k, &sum)| sum >= MARK_LIT * area && !near(k)).map(|(&k, _)| k).collect();
        lit.extend(marks);
    }
    lit
}

fn load(file: &str) -> fontdue::Font {
    let bytes = std::fs::read(format!("assets/fonts/{file}")).expect("a font in assets/fonts");
    fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).expect("a readable font")
}

/// The font's printable characters, sorted so the device can search them.
fn chars(font: &fontdue::Font) -> Vec<char> {
    let mut chars: Vec<char> = font.chars().keys().copied().filter(|c| !c.is_control()).collect();
    chars.sort();
    chars
}

/// `font` rasterised the ordinary way at `size`, smoothed, as `<name>_<size>`.
fn smooth_set(out: &mut String, font: &fontdue::Font, name: &str, size: u32) -> String {
    let px = size as f32;
    let line = font.horizontal_line_metrics(px).expect("horizontal metrics");
    let (mut glyphs, mut alpha) = (String::new(), Vec::new());
    for c in chars(font) {
        let (m, raster) = font.rasterize(c, px);
        writeln!(
            glyphs,
            "        Glyph {{ c: {c:?}, advance: {:?}, x: {}, y: {}, w: {}, h: {}, at: {} }},",
            m.advance_width.round(),
            m.xmin,
            m.ymin,
            m.width,
            m.height,
            alpha.len()
        )
        .unwrap();
        alpha.extend(raster);
    }
    font_data(out, &format!("{name}_{size}"), size, line.ascent.round() as i32, line.new_line_size.round() as i32, &glyphs, &alpha)
}

/// Writes one font at one size; returns a reference to it.
fn font_data(out: &mut String, set: &str, size: u32, ascent: i32, line: i32, glyphs: &str, alpha: &[u8]) -> String {
    writeln!(out, "static {set}_ALPHA: [u8; {}] = {alpha:?};", alpha.len()).unwrap();
    writeln!(
        out,
        "static {set}: FontData = FontData {{\n    size: {size},\n    ascent: {ascent},\n    line: {line},\n    glyphs: &[\n{glyphs}    ],\n    alpha: &{set}_ALPHA,\n}};"
    )
    .unwrap();
    format!("&{set}")
}
