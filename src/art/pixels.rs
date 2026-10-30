//! A tiny software canvas for pixel art: paint into a buffer pixel by
//! pixel, then send its pixels to the screen.

pub type Rgb = [u8; 3];

const BAYER: [[f32; 4]; 4] = [[0.0, 8.0, 2.0, 10.0], [12.0, 4.0, 14.0, 6.0], [3.0, 11.0, 1.0, 9.0], [15.0, 7.0, 13.0, 5.0]];

/// Ordered dithering: whether pixel (x, y) shows the next colour when a
/// blend is `frac` of the way there. This gives the checkered gradients of
/// 256-colour DOS art.
pub fn dither(x: i32, y: i32, frac: f32) -> bool {
    frac * 16.0 > BAYER[(y & 3) as usize][(x & 3) as usize] + 0.5
}

pub fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    [0, 1, 2].map(|i| (a[i] as f32 + (b[i] as f32 - a[i] as f32) * t).round() as u8)
}

pub fn scale(c: Rgb, k: f32) -> Rgb {
    c.map(|v| (v as f32 * k).min(255.0) as u8)
}

pub fn hash(s: &str) -> u64 {
    // FNV-1a: a stable seed from the country's name.
    s.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| (h ^ b as u64).wrapping_mul(0x0100_0000_01b3))
}

pub struct Canvas {
    w: i32,
    h: i32,
    px: Vec<Rgb>,
}

impl Canvas {
    pub fn new(w: i32, h: i32) -> Canvas {
        Canvas { w, h, px: vec![[0, 0, 0]; (w * h) as usize] }
    }

    pub fn set(&mut self, x: i32, y: i32, c: Rgb) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = c;
        }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<Rgb> {
        (x >= 0 && y >= 0 && x < self.w && y < self.h).then(|| self.px[(y * self.w + x) as usize])
    }

    pub fn ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, c: Rgb) {
        let (rx2, ry2) = ((rx * rx).max(1) as f32, (ry * ry).max(1) as f32);
        for y in -ry..=ry {
            for x in -rx..=rx {
                if (x * x) as f32 / rx2 + (y * y) as f32 / ry2 <= 1.0 {
                    self.set(cx + x, cy + y, c);
                }
            }
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgb) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, c);
            }
        }
    }

    pub fn disc(&mut self, cx: i32, cy: i32, r: i32, c: Rgb) {
        for y in -r..=r {
            for x in -r..=r {
                if x * x + y * y <= r * r + r / 2 {
                    self.set(cx + x, cy + y, c);
                }
            }
        }
    }

    /// A filled triangle pointing up: base centred on cx at y = base, apex h pixels above.
    pub fn peak(&mut self, cx: i32, base: i32, half: i32, h: i32, c: Rgb) {
        for i in 0..h {
            let w = half * (h - i) / h;
            self.rect(cx - w, base - i, 2 * w + 1, 1, c);
        }
    }

    /// A filled polygon (even-odd rule), for shapes like mountains and sails.
    pub fn poly(&mut self, points: &[(i32, i32)], c: Rgb) {
        let (top, bottom) = points.iter().fold((i32::MAX, i32::MIN), |(a, b), p| (a.min(p.1), b.max(p.1)));
        let mut xs = Vec::new();
        for y in top..=bottom {
            let yc = y as f32 + 0.5;
            xs.clear();
            for (i, &(x0, y0)) in points.iter().enumerate() {
                let (x1, y1) = points[(i + 1) % points.len()];
                let (fy0, fy1) = (y0 as f32, y1 as f32);
                if (fy0 <= yc) != (fy1 <= yc) {
                    xs.push(x0 as f32 + (yc - fy0) * (x1 - x0) as f32 / (fy1 - fy0));
                }
            }
            xs.sort_by(f32::total_cmp);
            for pair in xs.chunks_exact(2) {
                let (a, b) = (pair[0].round() as i32, pair[1].round() as i32);
                self.rect(a, y, b - a, 1, c);
            }
        }
    }

    /// A 1-pixel line (Bresenham).
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: Rgb) {
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
        let (mut x, mut y, mut err) = (x0, y0, dx + dy);
        loop {
            self.set(x, y, c);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Signs in a 3x5 pixel alphabet, 4 pixels per letter.
    pub fn text(&mut self, x: i32, y: i32, text: &str, c: Rgb) {
        for (n, ch) in text.chars().enumerate() {
            let Some(rows) = glyph(ch) else { continue };
            for (dy, row) in rows.iter().enumerate() {
                for (dx, bit) in row.chars().enumerate() {
                    if bit == '#' {
                        self.set(x + n as i32 * 4 + dx as i32, y + dy as i32, c);
                    }
                }
            }
        }
    }

    pub fn width(&self) -> i32 {
        self.w
    }

    pub fn height(&self) -> i32 {
        self.h
    }

    /// Row by row from the top left.
    pub fn pixels(&self) -> &[Rgb] {
        &self.px
    }
}

/// Width in pixels of `text` in the 3x5 alphabet.
pub fn text_width(text: &str) -> i32 {
    text.chars().count() as i32 * 4 - 1
}

/// The letters the signs need. Add more here if a sign wants them.
fn glyph(c: char) -> Option<[&'static str; 5]> {
    Some(match c {
        'A' => [".#.", "#.#", "###", "#.#", "#.#"],
        'B' => ["##.", "#.#", "##.", "#.#", "##."],
        'D' => ["##.", "#.#", "#.#", "#.#", "##."],
        'C' => [".##", "#..", "#..", "#..", ".##"],
        'E' => ["###", "#..", "##.", "#..", "###"],
        'F' => ["###", "#..", "##.", "#..", "#.."],
        'H' => ["#.#", "#.#", "###", "#.#", "#.#"],
        'I' => ["###", ".#.", ".#.", ".#.", "###"],
        'K' => ["#.#", "#.#", "##.", "#.#", "#.#"],
        'L' => ["#..", "#..", "#..", "#..", "###"],
        'M' => ["#.#", "###", "###", "#.#", "#.#"],
        'N' => ["##.", "#.#", "#.#", "#.#", "#.#"],
        'O' => ["###", "#.#", "#.#", "#.#", "###"],
        'R' => ["##.", "#.#", "##.", "#.#", "#.#"],
        'S' => [".##", "#..", ".#.", "..#", "##."],
        'T' => ["###", ".#.", ".#.", ".#.", ".#."],
        'U' => ["#.#", "#.#", "#.#", "#.#", "###"],
        'X' => ["#.#", "#.#", ".#.", "#.#", "#.#"],
        'Y' => ["#.#", "#.#", ".#.", ".#.", ".#."],
        _ => return None,
    })
}
