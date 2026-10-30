//! Pixel-art postcards of each city, standing in for the hand-drawn
//! pictures of the original. Each country has its own scene in
//! `landmarks.rs`, built from the parts here (sky, mountains, water, trees,
//! buildings and so on); a country without one gets a generic scene. The
//! time of day sets the sky and lighting, and lights windows after dark.

use crate::art::landmarks;
use crate::art::pixels::{Canvas, Rgb, dither, hash, mix, scale};
use crate::rules::rng::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeOfDay {
    Dawn,
    Day,
    Dusk,
    Night,
}

impl TimeOfDay {
    pub fn at(hour: i64) -> TimeOfDay {
        match hour {
            5..=7 => TimeOfDay::Dawn,
            8..=16 => TimeOfDay::Day,
            17..=19 => TimeOfDay::Dusk,
            _ => TimeOfDay::Night,
        }
    }

    /// Sky colours from the top down to the horizon.
    fn sky(self) -> [Rgb; 5] {
        match self {
            TimeOfDay::Dawn => [[40, 40, 110], [110, 70, 140], [200, 100, 120], [245, 150, 110], [255, 205, 150]],
            TimeOfDay::Day => [[60, 90, 220], [85, 115, 235], [110, 140, 245], [140, 170, 250], [180, 205, 255]],
            TimeOfDay::Dusk => [[30, 25, 80], [90, 40, 110], [180, 70, 90], [235, 120, 70], [255, 170, 90]],
            TimeOfDay::Night => [[5, 5, 25], [10, 12, 40], [18, 22, 60], [28, 35, 80], [40, 50, 95]],
        }
    }

    /// How brightly the land and buildings are lit.
    fn light(self) -> f32 {
        match self {
            TimeOfDay::Day => 1.0,
            TimeOfDay::Dawn | TimeOfDay::Dusk => 0.72,
            TimeOfDay::Night => 0.38,
        }
    }
}

pub const LAMP: Rgb = [255, 225, 120];

/// The postcard for a country (by English name) at `lat`, which varies the
/// generic scene.
pub fn paint(country: &str, lat: f64, time: TimeOfDay, w: i32, h: i32) -> Canvas {
    let mut p = Painter { cv: Canvas::new(w, h), w, h, time, rng: Rng::new(hash(country)) };
    if !landmarks::paint(&mut p, country) {
        generic(&mut p, lat);
    }
    p.cv
}

/// Draws with the time of day's lighting applied, plus the scene parts the
/// landmarks are built from. Use `cv` directly for light sources (lit
/// windows, lamps), which shouldn't darken at night.
pub struct Painter {
    pub cv: Canvas,
    pub w: i32,
    pub h: i32,
    pub time: TimeOfDay,
    pub rng: Rng,
}

impl Painter {
    pub fn lit(&self, c: Rgb) -> Rgb {
        let c = scale(c, self.time.light());
        match self.time {
            TimeOfDay::Dawn | TimeOfDay::Dusk => mix(c, [255, 130, 80], 0.08),
            TimeOfDay::Night => mix(c, [30, 40, 90], 0.12),
            TimeOfDay::Day => c,
        }
    }

    pub fn haze(&self) -> Rgb {
        self.time.sky()[4]
    }

    /// A colour seen from far off: it fades into the sky by `distance` (0 to 1).
    pub fn far(&self, c: Rgb, distance: f32) -> Rgb {
        mix(self.lit(c), self.haze(), distance)
    }

    pub fn after_dark(&self) -> bool {
        self.time != TimeOfDay::Day
    }

    pub fn unit(&mut self) -> f32 {
        self.rng.below(1000) as f32 / 1000.0
    }

    pub fn set(&mut self, x: i32, y: i32, c: Rgb) {
        let c = self.lit(c);
        self.cv.set(x, y, c);
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgb) {
        let c = self.lit(c);
        self.cv.rect(x, y, w, h, c);
    }

    pub fn disc(&mut self, x: i32, y: i32, r: i32, c: Rgb) {
        let c = self.lit(c);
        self.cv.disc(x, y, r, c);
    }

    pub fn ellipse(&mut self, x: i32, y: i32, rx: i32, ry: i32, c: Rgb) {
        let c = self.lit(c);
        self.cv.ellipse(x, y, rx, ry, c);
    }

    pub fn peak(&mut self, cx: i32, base: i32, half: i32, h: i32, c: Rgb) {
        let c = self.lit(c);
        self.cv.peak(cx, base, half, h, c);
    }

    pub fn poly(&mut self, points: &[(i32, i32)], c: Rgb) {
        let c = self.lit(c);
        self.cv.poly(points, c);
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: Rgb) {
        let c = self.lit(c);
        self.cv.line(x0, y0, x1, y1, c);
    }

    pub fn thick_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, t: i32, c: Rgb) {
        for dx in 0..t {
            for dy in 0..t {
                self.line(x0 + dx - t / 2, y0 + dy - t / 2, x1 + dx - t / 2, y1 + dy - t / 2, c);
            }
        }
    }

    /// Recolour pixels inside r that currently hold `from`, where `keep`
    /// says so: for shading one face of a shape after drawing it.
    pub fn recolour(&mut self, r: (i32, i32, i32, i32), from: Rgb, to: Rgb, keep: impl Fn(i32, i32) -> bool) {
        let (from, to) = (self.lit(from), self.lit(to));
        for y in r.1..r.1 + r.3 {
            for x in r.0..r.0 + r.2 {
                if self.cv.get(x, y) == Some(from) && keep(x, y) {
                    self.cv.set(x, y, to);
                }
            }
        }
    }

    // ---- the sky

    /// A dithered gradient reaching the horizon colour a little below
    /// `horizon` (and filling the whole picture, so gaps between buildings
    /// show sky), with the sun (high by day, low and big at dawn and dusk)
    /// or the moon and stars.
    pub fn sky(&mut self, horizon: i32, sun_x: i32) {
        let sky = self.time.sky();
        let bottom = horizon + 40;
        for y in 0..self.h {
            let v = (y.min(bottom) as f32 / bottom as f32) * (sky.len() - 1) as f32;
            let band = (v as usize).min(sky.len() - 2);
            for x in 0..self.w {
                let c = if dither(x, y, v - band as f32) { sky[band + 1] } else { sky[band] };
                self.cv.set(x, y, c);
            }
        }
        match self.time {
            TimeOfDay::Day => self.cv.disc(sun_x, self.h / 8, 10, [255, 245, 170]),
            TimeOfDay::Dawn | TimeOfDay::Dusk => self.cv.disc(sun_x, horizon - 16, 15, [255, 190, 100]),
            TimeOfDay::Night => {
                for _ in 0..70 {
                    let (x, y) = (self.rng.below(self.w as usize) as i32, self.rng.below(horizon.max(1) as usize) as i32);
                    let c = if self.rng.below(3) == 0 { [255, 255, 200] } else { [150, 160, 200] };
                    self.cv.set(x, y, c);
                }
                self.cv.disc(sun_x, self.h / 9, 8, [235, 235, 215]);
                self.cv.disc(sun_x + 4, self.h / 9 - 2, 7, sky[0]);
            }
        }
    }

    /// Puffy clouds between y0 and y1, white by day and dim at night.
    pub fn clouds(&mut self, n: usize, y0: i32, y1: i32) {
        for _ in 0..n {
            let x = self.rng.below(self.w as usize) as i32;
            let y = y0 + self.rng.below((y1 - y0).max(1) as usize) as i32;
            let size = 8 + self.rng.below(10) as i32;
            for i in 0..4 {
                let (dx, r) = (i * size * 2 / 3 - size, size - (i - 1).abs() * 2);
                self.ellipse(x + dx, y - (r / 3), r, r * 2 / 3, [245, 245, 250]);
            }
            let under = self.lit([200, 205, 220]);
            for xx in x - size - 4..x + size * 2 {
                for yy in y..y + size / 3 {
                    if self.cv.get(xx, yy) == Some(self.lit([245, 245, 250])) && dither(xx, yy, 0.5) {
                        self.cv.set(xx, yy, under);
                    }
                }
            }
        }
    }

    pub fn birds(&mut self, n: usize, y0: i32, y1: i32) {
        for _ in 0..n {
            let x = self.rng.below(self.w as usize) as i32;
            let y = y0 + self.rng.below((y1 - y0).max(1) as usize) as i32;
            for (dx, dy) in [(-3, -1), (-2, -1), (-1, 0), (0, 1), (1, 0), (2, -1), (3, -1)] {
                self.set(x + dx, y + dy, [40, 40, 50]);
            }
        }
    }

    // ---- land and water

    fn wave_heights(&mut self, bumps: f32) -> Vec<f32> {
        let waves: Vec<(f32, f32, f32)> = (0..3)
            .map(|i| {
                let freq = bumps * (1.0 + i as f32 * 1.7) / self.w as f32 * std::f32::consts::TAU;
                let phase = self.unit() * std::f32::consts::TAU;
                (freq, phase, 1.0 / (1.0 + i as f32))
            })
            .collect();
        let total: f32 = waves.iter().map(|w| w.2).sum();
        (0..self.w).map(|x| waves.iter().map(|(f, p, a)| ((x as f32 * f + p).sin() * 0.5 + 0.5) * a).sum::<f32>() / total).collect()
    }

    /// Distant mountains rising up to `height` above `base`, faded by
    /// `distance`, with snow on the peaks if asked. Returns each column's top.
    pub fn mountains(&mut self, base: i32, height: i32, bumps: f32, rock: Rgb, distance: f32, snow: bool) -> Vec<i32> {
        let heights = self.wave_heights(bumps);
        let colour = self.far(rock, distance);
        let white = self.far([250, 250, 255], distance * 0.6);
        let mut tops = Vec::new();
        for x in 0..self.w {
            let v = heights[x as usize];
            let top = base - (v * height as f32) as i32;
            tops.push(top);
            for y in top..self.h {
                let snowy = snow && v > 0.6 && y < top + 3 + ((v - 0.6) * height as f32 * 0.6) as i32 + x * 7 % 3;
                self.cv.set(x, y, if snowy { white } else { colour });
            }
        }
        tops
    }

    /// Rolling hills in a nearer colour, shaded darker further down.
    pub fn hills(&mut self, base: i32, height: i32, bumps: f32, colour: Rgb) -> Vec<i32> {
        let heights = self.wave_heights(bumps);
        let mut tops = Vec::new();
        for x in 0..self.w {
            let top = base - (heights[x as usize] * height as f32) as i32;
            tops.push(top);
            for y in top..self.h {
                let c = if dither(x, y, ((y - top) as f32 / 50.0).min(0.7)) { scale(colour, 0.82) } else { colour };
                self.set(x, y, c);
            }
        }
        tops
    }

    /// Water from top to bottom: lighter towards the horizon, with ripples.
    pub fn water(&mut self, top: i32, bottom: i32, colour: Rgb) {
        let horizon = mix(colour, self.haze(), 0.35);
        for y in top..bottom.min(self.h) {
            let t = (y - top) as f32 / (bottom - top).max(1) as f32;
            for x in 0..self.w {
                let c = if dither(x, y, t) { scale(colour, 0.85) } else { horizon };
                self.set(x, y, c);
            }
            if (y - top) % 3 == 1 {
                for _ in 0..self.w / 16 {
                    let x = self.rng.below(self.w as usize) as i32;
                    let len = 2 + self.rng.below(6) as i32 + (y - top) / 10;
                    self.rect(x, y, len, 1, mix(colour, [240, 245, 255], 0.5));
                }
            }
        }
    }

    /// Ground from `top` to the bottom, speckled.
    pub fn ground(&mut self, top: i32, colour: Rgb) {
        for y in top..self.h {
            for x in 0..self.w {
                let speckle = dither(x, y, 0.3) && (x * 13 + y * 7) % 11 == 0;
                self.set(x, y, if speckle { scale(colour, 0.82) } else { colour });
            }
        }
    }

    /// Grass that thins into tufts: savanna, meadows.
    pub fn grass(&mut self, top: i32, colour: Rgb) {
        self.ground(top, colour);
        for _ in 0..self.w * (self.h - top) / 60 {
            let x = self.rng.below(self.w as usize) as i32;
            let y = top + self.rng.below((self.h - top).max(1) as usize) as i32;
            let tall = 1 + (y - top) / 25;
            self.rect(x, y - tall, 1, tall + 1, scale(colour, 0.7));
            self.set(x + 1, y - tall + 1, scale(colour, 1.15));
        }
    }

    // ---- buildings

    /// A grid of windows: dark by day, some lit after dark.
    pub fn windows(&mut self, area: (i32, i32, i32, i32), step: (i32, i32), size: (i32, i32), wall: Rgb) {
        let (x, y, w, h) = area;
        let mut wy = y;
        while wy + size.1 <= y + h {
            let mut wx = x;
            while wx + size.0 <= x + w {
                if self.after_dark() && self.rng.below(100) < 55 {
                    self.cv.rect(wx, wy, size.0, size.1, LAMP);
                } else {
                    self.rect(wx, wy, size.0, size.1, scale(wall, 0.5));
                }
                wx += step.0;
            }
            wy += step.1;
        }
    }

    /// A row of simple buildings standing on `base`, faded by `distance`.
    pub fn buildings(&mut self, x0: i32, x1: i32, base: i32, heights: (i32, i32), walls: &[Rgb], distance: f32) {
        let mut x = x0;
        while x < x1 {
            let bw = 8 + self.rng.below(16) as i32;
            let tall = heights.0 + self.rng.below((heights.1 - heights.0).max(1) as usize) as i32;
            let wall = walls[self.rng.below(walls.len())];
            let (c, side) = (self.far(wall, distance), self.far(scale(wall, 0.75), distance));
            self.cv.rect(x, base - tall, bw, tall, c);
            self.cv.rect(x + bw - 2, base - tall, 2, tall, side);
            if distance < 0.5 || self.after_dark() {
                for wy in (base - tall + 3..base - 3).step_by(5) {
                    for wx in (x + 2..x + bw - 3).step_by(4) {
                        let on = self.after_dark() && self.rng.below(100) < 45;
                        let wc = if on { LAMP } else { self.far(scale(wall, 0.55), distance) };
                        self.cv.rect(wx, wy, 2, 2, wc);
                    }
                }
            }
            x += bw + self.rng.below(2) as i32;
        }
    }

    // ---- trees, people and animals

    pub fn palm(&mut self, x: i32, base: i32, height: i32) {
        for i in 0..height {
            let ring = if i % 3 == 0 { [95, 62, 35] } else { [130, 88, 50] };
            self.rect(x + i * i / (height * 6), base - i, 3, 1, ring);
        }
        let (fx, fy) = ((x + height / 6 + 1) as f32, (base - height) as f32);
        let len = 10 + height / 4;
        for (dx, lift) in [(-1.0, 0.9), (1.0, 0.9), (-1.0, 0.3), (1.0, 0.3), (-0.5, 1.4), (0.5, 1.4)] {
            for i in 0..len {
                let t = i as f32;
                let (px, py) = ((fx + dx * t) as i32, (fy - lift * t + 0.09 * t * t) as i32);
                self.rect(px, py, 2, 1, [45, 150, 55]);
                if i % 2 == 0 {
                    self.set(px, py + 1, [25, 105, 40]);
                }
            }
        }
        self.rect(fx as i32 - 1, fy as i32 + 1, 3, 2, [110, 75, 40]);
    }

    /// A broadleaf tree: a trunk and a lumpy, shaded crown of radius r.
    pub fn tree(&mut self, x: i32, base: i32, r: i32, leaf: Rgb) {
        self.rect(x - 1, base - r, 3, r, [90, 60, 40]);
        for (dx, dy, rr) in [(0, -r - r / 2, r), (-r / 2, -r - r / 4, r * 3 / 4), (r / 2, -r - r / 4, r * 3 / 4)] {
            self.disc(x + dx, base + dy, rr, leaf);
        }
        let light = scale(leaf, 1.25);
        self.disc(x - r / 3, base - r * 2 + r / 4, (r / 3).max(1), light);
    }

    pub fn conifer(&mut self, x: i32, base: i32, h: i32, leaf: Rgb) {
        self.rect(x - 1, base - h / 6, 2, h / 6, [80, 55, 40]);
        for i in 0..3 {
            let tier_base = base - h / 6 - i * h / 4;
            self.peak(x, tier_base, h / 4 - i * h / 14, h / 3, leaf);
        }
    }

    /// A tall, narrow Mediterranean cypress.
    pub fn cypress(&mut self, x: i32, base: i32, h: i32) {
        self.ellipse(x, base - h / 2, (h / 9).max(2), h / 2, [30, 80, 45]);
        self.ellipse(x - 1, base - h / 2 - 2, (h / 18).max(1), h / 2 - 4, [50, 105, 60]);
    }

    /// A flat-topped African acacia.
    pub fn acacia(&mut self, x: i32, base: i32, r: i32) {
        self.line(x, base, x - r / 4, base - r, [80, 55, 35]);
        self.line(x + 1, base, x - r / 4 + 1, base - r, [80, 55, 35]);
        self.line(x - r / 4, base - r, x - r / 2, base - r - r / 3, [80, 55, 35]);
        self.line(x - r / 4, base - r, x + r / 3, base - r - r / 3, [80, 55, 35]);
        self.ellipse(x - r / 8, base - r - r / 2, r, (r / 4).max(2), [70, 110, 45]);
        self.ellipse(x - r / 8, base - r - r / 2 - 1, r - 2, (r / 6).max(1), [95, 135, 60]);
    }

    /// A small standing figure.
    pub fn person(&mut self, x: i32, base: i32, clothes: Rgb) {
        self.rect(x - 1, base - 4, 1, 4, [40, 40, 50]);
        self.rect(x + 1, base - 4, 1, 4, [40, 40, 50]);
        self.rect(x - 2, base - 10, 5, 6, clothes);
        self.disc(x, base - 12, 2, [170, 120, 90]);
    }
}

/// A scene for a country without its own: mountains, maybe water, a
/// skyline and trees, varied by latitude.
fn generic(p: &mut Painter, lat: f64) {
    let (w, h) = (p.w, p.h);
    let lat = lat.abs();
    let tropical = lat < 15.0;
    let cold = lat > 55.0;
    let horizon = h * 52 / 100;
    let sun_x = 30 + p.rng.below((w - 60) as usize) as i32;
    p.sky(horizon, sun_x);
    let snow = cold || (lat > 30.0 && p.rng.below(2) == 0);
    let height = 25 + p.rng.below(50) as i32;
    p.mountains(horizon, height, 2.5, [100, 100, 125], 0.45, snow);
    let hill = if tropical { [30, 120, 55] } else { [70, 140, 65] };
    p.hills(horizon + 6, 22, 1.5, hill);
    let ground = h * 74 / 100;
    if p.rng.below(2) == 0 {
        p.water(horizon + 14, ground, [60, 100, 190]);
    }
    let walls: [Rgb; 3] =
        if tropical { [[240, 200, 170], [200, 230, 220], [250, 248, 238]] } else { [[160, 160, 170], [180, 115, 95], [205, 195, 175]] };
    p.buildings(-4, w, ground, (18, 60), &walls, 0.0);
    p.ground(ground, [120, 120, 125]);
    let mut x = p.rng.below(30) as i32;
    while x < w {
        let base = ground + 10 + p.rng.below((h - ground - 14).max(1) as usize) as i32;
        if tropical {
            p.palm(x, base, 30);
        } else if cold {
            p.conifer(x, base, 24, [30, 80, 55]);
        } else {
            p.tree(x, base, 7, [45, 110, 50]);
        }
        x += 50 + p.rng.below(60) as i32;
    }
}
