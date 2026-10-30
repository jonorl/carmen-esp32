//! One postcard scene per country, each built around something the place
//! is known for. Coordinates are for the 274x282 picture frame. Every
//! colour goes through the painter's lighting, so the same scene works at
//! dawn, by day, at dusk and at night.
//!
//! To add a country, add a function and a line in `paint`.

use crate::art::pixels::{Rgb, dither, mix, scale};
use crate::art::scene::{LAMP, Painter, TimeOfDay};

/// Paint the scene for `country`. False if there isn't one.
pub fn paint(p: &mut Painter, country: &str) -> bool {
    let scene: fn(&mut Painter) = match country {
        "Kenya" => kenya,
        "Uganda" => uganda,
        "Egypt" => egypt,
        "South Africa" => south_africa,
        "Tanzania" => tanzania,
        "United Kingdom" => united_kingdom,
        "France" => france,
        "Italy" => italy,
        "Spain" => spain,
        "Greece" => greece,
        "Türkiye" => turkiye,
        "Norway" => norway,
        "Netherlands" => netherlands,
        "Germany" => germany,
        "Japan" => japan,
        "Nepal" => nepal,
        "United States" => united_states,
        "Mexico" => mexico,
        "Brazil" => brazil,
        "Argentina" => argentina,
        "Peru" => peru,
        "Australia" => australia,
        "Venezuela" => venezuela,
        "Ecuador" => ecuador,
        "Democratic Republic of the Congo" => congo,
        "Ireland" => ireland,
        "India" => india,
        "Kyrgyzstan" => kyrgyzstan,
        "Mauritius" => mauritius,
        "Yemen" => yemen,
        "Belgium" => belgium,
        "Uruguay" => uruguay,
        "Monkey Island" => monkey_island,
        _ => return false,
    };
    scene(p);
    true
}

const WHITE: Rgb = [242, 242, 236];
const MARBLE: Rgb = [235, 230, 218];
const SAND: Rgb = [226, 196, 140];
const ROCK: Rgb = [120, 118, 132];
const GRASS: Rgb = [70, 140, 60];
const SEA: Rgb = [50, 100, 180];
const TERRACOTTA: Rgb = [190, 85, 55];
const DARK: Rgb = [40, 40, 50];

// ------------------------------------------------------------------ Africa

fn kenya(p: &mut Painter) {
    // Savanna below the Ngong Hills, giraffes and acacias, Nairobi's
    // skyline and its round KICC tower on the horizon.
    p.sky(150, 200);
    p.mountains(152, 20, 1.2, [90, 100, 120], 0.5, false);
    p.buildings(150, 262, 154, (6, 22), &[[205, 205, 210], [175, 180, 190], [220, 215, 200]], 0.45);
    let tower = p.far([200, 190, 170], 0.4);
    p.cv.rect(198, 122, 9, 32, tower);
    p.cv.ellipse(202, 121, 8, 2, tower);
    p.cv.rect(202, 110, 1, 10, tower);
    p.grass(154, [205, 170, 85]);
    p.acacia(232, 176, 14);
    p.acacia(150, 168, 8);
    p.acacia(48, 214, 28);
    giraffe(p, 196, 214, 0.9, false);
    giraffe(p, 118, 270, 1.5, true);
}

fn giraffe(p: &mut Painter, x: i32, base: i32, s: f32, left: bool) {
    let d = if left { -1 } else { 1 };
    let k = |v: f32| (v * s) as i32;
    let coat: Rgb = [232, 178, 82];
    let patch: Rgb = [112, 62, 28];
    let body_y = base - k(28.0);
    for dx in [-8.0, -4.0, 5.0, 9.0] {
        p.rect(x + d * k(dx), body_y + k(3.0), (k(2.0)).max(1), base - body_y - k(3.0), coat);
    }
    p.ellipse(x, body_y, k(11.0), k(6.0), coat);
    let (nx, ny) = (x + d * k(8.0), body_y - k(3.0));
    let (hx, hy) = (x + d * k(17.0), body_y - k(32.0));
    p.thick_line(nx, ny, hx, hy, k(3.0).max(2), coat);
    p.ellipse(hx + d * k(3.0), hy, k(4.0).max(2), k(2.0).max(1), coat);
    p.set(hx, hy - k(3.0), patch);
    p.set(hx + d, hy - k(3.0), patch);
    p.line(x - d * k(11.0), body_y, x - d * k(13.0), body_y + k(10.0), patch);
    for (dx, dy) in [(-6, -2), (-1, 1), (4, -2), (8, 2), (-3, -4), (2, 3)] {
        p.disc(x + d * k(dx as f32), body_y + k(dy as f32), k(1.6).max(1), patch);
    }
    for i in 1..5 {
        let t = i as f32 / 5.0;
        let (px, py) = (nx as f32 + (hx - nx) as f32 * t, ny as f32 + (hy - ny) as f32 * t);
        p.set(px as i32, py as i32, patch);
    }
}

fn uganda(p: &mut Painter) {
    // Kampala's red-roofed houses on green hills above Lake Victoria, with
    // a crested crane (the bird on the flag) and banana plants on the shore.
    p.sky(128, 60);
    p.mountains(132, 22, 1.5, [70, 110, 90], 0.45, false);
    let tops = p.hills(160, 34, 2.2, [55, 140, 60]);
    for _ in 0..34 {
        let x = p.rng.below(p.w as usize) as i32;
        let y = tops[x as usize] + 3 + p.rng.below(18) as i32;
        p.rect(x, y, 7, 4, [235, 230, 220]);
        p.rect(x - 1, y - 2, 9, 2, TERRACOTTA);
        if p.after_dark() && p.rng.below(2) == 0 {
            p.cv.rect(x + 2, y + 1, 2, 2, LAMP);
        }
    }
    p.water(186, 236, [70, 110, 170]);
    p.rect(150, 206, 24, 3, [110, 70, 40]);
    p.person(160, 206, [200, 60, 40]);
    p.grass(236, [60, 135, 50]);
    for x in [12, 246] {
        banana(p, x, 268);
    }
    crane(p, 80, 276);
}

fn banana(p: &mut Painter, x: i32, base: i32) {
    p.rect(x - 2, base - 30, 5, 30, [100, 130, 60]);
    for (dx, dy) in [(-18, -6), (18, -6), (-14, 6), (14, 6), (-4, -14)] {
        p.thick_line(x, base - 30, x + dx, base - 34 + dy, 3, [60, 150, 60]);
        p.line(x, base - 31, x + dx, base - 35 + dy, [90, 180, 80]);
    }
}

fn crane(p: &mut Painter, x: i32, base: i32) {
    // A grey crowned crane: long legs, grey body, black head, golden crown.
    p.line(x - 2, base, x - 1, base - 18, DARK);
    p.line(x + 3, base, x + 2, base - 18, DARK);
    p.ellipse(x, base - 24, 9, 6, [150, 150, 160]);
    p.ellipse(x - 4, base - 23, 5, 3, WHITE);
    p.rect(x - 11, base - 24, 4, 3, DARK);
    p.rect(x + 5, base - 42, 2, 16, [150, 150, 160]);
    p.disc(x + 7, base - 44, 3, DARK);
    p.set(x + 7, base - 43, WHITE);
    p.set(x + 8, base - 41, [210, 40, 40]);
    p.rect(x + 10, base - 44, 3, 1, [60, 60, 60]);
    for (dx, dy) in [(4, -50), (6, -51), (8, -50), (5, -49), (7, -49), (9, -48), (3, -48)] {
        p.set(x + dx, base + dy, [250, 205, 60]);
    }
}

fn egypt(p: &mut Painter) {
    // The pyramids of Giza and the Sphinx, with Cairo's minarets far off.
    p.sky(162, 214);
    p.buildings(0, 70, 164, (6, 16), &[[220, 205, 175], [200, 185, 150]], 0.5);
    let minaret = p.far([220, 205, 175], 0.5);
    for x in [18, 46] {
        p.cv.rect(x, 136, 3, 28, minaret);
        p.cv.peak(x + 1, 136, 2, 6, minaret);
    }
    p.ground(162, SAND);
    for y in (170..p.h).step_by(9) {
        let x0 = p.rng.below(p.w as usize) as i32;
        let len = 30 + p.rng.below(40) as i32;
        p.rect(x0, y, len, 1, scale(SAND, 0.88));
    }
    pyramid(p, 85, 200, 48, 60);
    pyramid(p, 168, 206, 72, 90);
    pyramid(p, 236, 198, 26, 32);
    // The Sphinx, facing right.
    let lion: Rgb = [205, 170, 115];
    p.rect(30, 232, 46, 12, lion);
    p.rect(74, 238, 14, 6, lion);
    p.rect(66, 214, 14, 20, lion);
    p.rect(62, 216, 4, 14, scale(lion, 0.8));
    p.rect(70, 220, 9, 8, scale(lion, 0.9));
    p.set(76, 222, DARK);
    p.palm(248, 274, 44);
    p.palm(222, 266, 34);
}

fn pyramid(p: &mut Painter, cx: i32, base: i32, half: i32, h: i32) {
    let (lit, shade) = ([228, 198, 142], [182, 150, 100]);
    p.poly(&[(cx - half, base), (cx, base - h), (cx, base)], lit);
    p.poly(&[(cx, base), (cx, base - h), (cx + half, base)], shade);
    for y in (base - h + 4..base).step_by(4) {
        let w = half * (y - (base - h)) / h;
        p.rect(cx - w, y, w, 1, scale(lit, 0.9));
        p.rect(cx, y, w, 1, scale(shade, 0.9));
    }
}

fn south_africa(p: &mut Painter) {
    // Table Mountain with its "tablecloth" of cloud, Lion's Head beside it,
    // Cape Town at its foot and a yacht on Table Bay.
    p.sky(170, 50);
    let mountain = [
        (0, 175),
        (0, 128),
        (22, 104),
        (44, 84),
        (58, 96),
        (70, 92),
        (190, 90),
        (204, 112),
        (210, 136),
        (222, 116),
        (236, 102),
        (250, 110),
        (262, 134),
        (274, 150),
        (274, 175),
    ];
    p.poly(&mountain, ROCK);
    for x in (72..200).step_by(5) {
        let len = 10 + p.rng.below(26) as i32;
        p.rect(x, 93, 1, len, scale(ROCK, 0.8));
    }
    p.poly(&[(0, 175), (0, 150), (60, 140), (130, 150), (200, 146), (274, 158), (274, 175)], [70, 120, 70]);
    if matches!(p.time, TimeOfDay::Day | TimeOfDay::Dawn) {
        for x in 62..198 {
            let drip = if x % 11 < 5 { 4 + (x * 7) % 6 } else { 1 };
            p.rect(x, 82, 1, 10 + drip - 2, [245, 245, 250]);
            if dither(x, 90 + drip, 0.5) {
                p.set(x, 90 + drip, [220, 222, 232]);
            }
        }
    }
    p.buildings(0, 274, 192, (6, 20), &[[240, 240, 235], [220, 220, 215], [235, 225, 200]], 0.15);
    p.water(192, 240, SEA);
    sailboat(p, 182, 226, 24, WHITE);
    p.ground(240, [225, 205, 160]);
    for x in [20, 60, 230] {
        p.disc(x, 252, 5, [90, 120, 70]);
    }
}

fn sailboat(p: &mut Painter, x: i32, waterline: i32, size: i32, sail: Rgb) {
    p.poly(&[(x - size / 2, waterline - 4), (x + size / 2, waterline - 4), (x + size / 3, waterline), (x - size / 3, waterline)], WHITE);
    p.line(x, waterline - 4, x, waterline - 4 - size * 3 / 2, DARK);
    p.poly(&[(x + 1, waterline - 6 - size * 3 / 2), (x + 1, waterline - 6), (x + size / 2 + 2, waterline - 6)], sail);
    p.poly(&[(x - 1, waterline - 8 - size), (x - 1, waterline - 6), (x - size / 3, waterline - 6)], sail);
}

fn tanzania(p: &mut Painter) {
    // The Indian Ocean at Dar es Salaam: a dhow with its lateen sail,
    // white sand and coconut palms.
    p.sky(150, 220);
    p.buildings(0, 96, 152, (5, 14), &[[240, 240, 235], [225, 220, 205]], 0.45);
    p.water(150, 212, [40, 150, 190]);
    dhow(p, 138, 190, 1.0);
    dhow(p, 228, 164, 0.5);
    p.ground(212, [238, 224, 186]);
    p.rect(0, 212, p.w, 3, [205, 190, 150]);
    for (x, y) in [(120, 236), (180, 258), (90, 270)] {
        p.set(x, y, [240, 200, 200]);
    }
    p.palm(28, 274, 52);
    p.palm(64, 258, 38);
    p.palm(250, 268, 46);
}

fn dhow(p: &mut Painter, x: i32, waterline: i32, s: f32) {
    let k = |v: f32| (v * s) as i32;
    p.poly(
        &[(x - k(30.0), waterline - k(8.0)), (x + k(28.0), waterline - k(6.0)), (x + k(20.0), waterline), (x - k(22.0), waterline)],
        [120, 78, 44],
    );
    p.rect(x - k(26.0), waterline - k(8.0), k(52.0), 1, [160, 110, 60]);
    p.line(x - k(2.0), waterline - k(8.0), x - k(8.0), waterline - k(62.0), [80, 55, 35]);
    p.poly(&[(x - k(8.0), waterline - k(62.0)), (x + k(34.0), waterline - k(12.0)), (x - k(26.0), waterline - k(14.0))], [245, 238, 220]);
    p.line(x - k(8.0), waterline - k(62.0), x + k(34.0), waterline - k(12.0), [200, 190, 170]);
}

fn congo(p: &mut Painter) {
    // The wide Congo River at Kinshasa: pirogues with standing paddlers,
    // rainforest on the banks, Brazzaville faint on the far shore.
    p.sky(132, 190);
    p.buildings(60, 220, 140, (3, 10), &[[210, 205, 195], [190, 185, 175]], 0.55);
    p.water(140, 238, [110, 105, 70]);
    for _ in 0..14 {
        let (x, y) = (p.rng.below(p.w as usize) as i32, 150 + p.rng.below(80) as i32);
        p.ellipse(x, y, 4, 1, [70, 120, 50]);
    }
    pirogue(p, 70, 196, 54);
    pirogue(p, 186, 168, 32);
    for (x0, x1) in [(-10, 64), (214, 290)] {
        for i in 0..14 {
            let x = x0 + (x1 - x0) * i / 14;
            let top = 110 + p.rng.below(60) as i32;
            p.rect(x, top + 10, 3, 240 - top, [70, 50, 35]);
            let g = [30 + p.rng.below(30) as u8, 90 + p.rng.below(50) as u8, 40];
            let r = 10 + p.rng.below(6) as i32;
            p.disc(x + 1, top, r, g);
        }
    }
    p.ground(238, [100, 80, 55]);
    for x in (0..p.w).step_by(4) {
        let tall = 6 + (x * 7) % 9;
        p.rect(x, 238 - tall, 1, tall, [80, 130, 60]);
    }
}

fn pirogue(p: &mut Painter, x: i32, waterline: i32, len: i32) {
    p.poly(
        &[(x - len / 2, waterline - 3), (x + len / 2, waterline - 3), (x + len / 2 - 4, waterline), (x - len / 2 + 4, waterline)],
        [70, 45, 30],
    );
    p.person(x + len / 5, waterline - 3, [220, 180, 40]);
    p.line(x + len / 5 + 3, waterline - 14, x + len / 5 + 9, waterline + 2, [120, 90, 50]);
    if len > 40 {
        p.person(x - len / 4, waterline - 3, [60, 120, 200]);
    }
}

fn mauritius(p: &mut Painter) {
    // A turquoise lagoon inside the reef, with Le Morne Brabant's great
    // rock, white sand, palms and a pirogue under sail.
    p.sky(150, 70);
    p.poly(&[(140, 154), (168, 114), (192, 94), (226, 88), (248, 100), (264, 154)], [70, 95, 80]);
    p.poly(&[(192, 94), (226, 88), (248, 100), (236, 124), (204, 120)], [95, 105, 100]);
    p.water(150, 216, [40, 175, 190]);
    for x in 0..p.w {
        if dither(x, 162, 0.6) {
            p.set(x, 162, WHITE);
        }
        if x % 3 == 0 {
            p.set(x, 163, WHITE);
        }
    }
    for y in 190..216 {
        for x in 0..p.w {
            if dither(x, y, (y - 190) as f32 / 26.0) {
                p.set(x, y, [120, 225, 210]);
            }
        }
    }
    sailboat(p, 112, 196, 22, [250, 200, 60]);
    p.ground(216, [246, 238, 214]);
    p.palm(24, 268, 58);
    p.palm(58, 250, 42);
}

// ------------------------------------------------------------------ Europe

fn united_kingdom(p: &mut Painter) {
    // Big Ben and the Houses of Parliament over the Thames, with a red bus
    // crossing Westminster Bridge.
    p.sky(170, 236);
    if !p.after_dark() {
        p.clouds(6, 20, 110);
    }
    let stone: Rgb = [205, 180, 125];
    p.rect(0, 168, 198, 64, stone);
    for x in (2..198).step_by(9) {
        p.rect(x, 168, 2, 64, scale(stone, 0.78));
        p.peak(x + 1, 168, 2, 8, scale(stone, 0.85));
    }
    p.windows((5, 176, 190, 46), (9, 11), (3, 6), stone);
    // The clock tower.
    p.rect(204, 72, 26, 160, stone);
    p.rect(226, 72, 4, 160, scale(stone, 0.78));
    p.rect(206, 76, 22, 22, [225, 210, 160]);
    let face = if p.after_dark() { LAMP } else { WHITE };
    p.cv.disc(217, 87, 8, face);
    p.line(217, 87, 217, 81, DARK);
    p.line(217, 87, 221, 89, DARK);
    p.rect(204, 60, 26, 12, stone);
    for x in (206..228).step_by(5) {
        p.rect(x, 62, 2, 8, scale(stone, 0.6));
    }
    p.peak(217, 60, 14, 30, [70, 95, 90]);
    p.rect(217, 22, 1, 10, [200, 180, 90]);
    for x in [204, 229] {
        p.peak(x, 60, 2, 8, stone);
    }
    p.water(232, p.h, [80, 100, 120]);
    p.rect(0, 226, p.w, 7, [60, 110, 80]);
    for x in (0..p.w).step_by(34) {
        p.ellipse(x + 17, 236, 14, 4, [60, 110, 80]);
    }
    // A double-decker bus on the bridge.
    p.rect(36, 208, 38, 18, [200, 30, 30]);
    p.rect(36, 216, 38, 1, [150, 20, 20]);
    p.windows((39, 210, 33, 4), (6, 6), (4, 3), [200, 30, 30]);
    p.windows((39, 218, 33, 4), (6, 6), (4, 3), [200, 30, 30]);
    for x in [44, 66] {
        p.disc(x, 226, 3, DARK);
    }
}

fn france(p: &mut Painter) {
    // The Eiffel Tower above the Seine and Paris rooftops.
    p.sky(190, 60);
    let cream: Rgb = [228, 214, 182];
    let mut x = -6;
    while x < p.w {
        let bw = 20 + p.rng.below(14) as i32;
        let tall = 22 + p.rng.below(14) as i32;
        p.rect(x, 216 - tall, bw, tall, cream);
        p.rect(x - 1, 216 - tall - 5, bw + 2, 5, [110, 115, 130]);
        p.windows((x + 3, 216 - tall + 3, bw - 5, tall - 6), (5, 7), (2, 4), cream);
        x += bw + 1;
    }
    tower(p, 137, 230, 22);
    for x in (6..p.w).step_by(22) {
        p.tree(x, 238, 7, [55, 120, 55]);
    }
    p.water(238, p.h, [70, 100, 140]);
    p.rect(0, 262, p.w, 5, [190, 180, 160]);
    for x in (10..p.w).step_by(40) {
        p.ellipse(x + 10, 268, 12, 4, [190, 180, 160]);
    }
}

/// The Eiffel Tower: two curving legs meeting at the top, an arch between
/// the legs, two platforms and a lattice texture. Golden lights at night.
fn tower(p: &mut Painter, cx: i32, base: i32, top: i32) {
    let metal: Rgb = if p.after_dark() { [210, 160, 80] } else { [120, 92, 64] };
    let span = (base - top) as f32;
    let first = base - 56;
    let half_at = |y: i32| 3.0 + 60.0 * ((y - top) as f32 / span).powf(2.3);
    for y in top..base {
        let half = half_at(y) as i32;
        let opening = if y > first {
            let t = ((y - first) as f32 / (base - first) as f32).sqrt();
            ((half - 9) as f32 * t) as i32
        } else {
            -1
        };
        for x in cx - half..=cx + half {
            if (x - cx).abs() <= opening {
                continue;
            }
            let lattice = (x + y) % 4 == 0 || (x - y).rem_euclid(4) == 0;
            let c = if lattice { scale(metal, 0.7) } else { metal };
            p.set(x, y, c);
        }
    }
    for (y, extra) in [(first, 4), (base - 112, 2)] {
        let half = half_at(y) as i32 + extra;
        p.rect(cx - half, y - 3, half * 2 + 1, 4, scale(metal, 0.8));
    }
    p.rect(cx - 2, top - 4, 5, 5, scale(metal, 0.8));
    p.rect(cx, top - 14, 1, 10, scale(metal, 0.8));
    if p.after_dark() {
        for _ in 0..40 {
            let y = top + p.rng.below((base - top) as usize) as i32;
            let half = half_at(y) as i32;
            let x = cx - half + p.rng.below((half * 2 + 1).max(1) as usize) as i32;
            p.cv.set(x, y, [255, 250, 220]);
        }
    }
}

fn italy(p: &mut Painter) {
    // Milan's Duomo: a white marble Gothic front bristling with pinnacles,
    // the forest of spires on the roof behind, and the golden Madonnina on
    // the tallest, over a square full of pigeons.
    p.sky(170, 40);
    p.clouds(2, 24, 60);
    p.buildings(0, p.w, 226, (40, 60), &[[200, 170, 130], [220, 190, 150], [190, 150, 120]], 0.4);
    let marble: Rgb = [236, 226, 216];
    let shade = scale(marble, 0.8);
    // The roof's spires, and the central one with the Madonnina.
    for x in (60..220).step_by(12) {
        pinnacle(p, x, 140, 104 + (x * 7 % 13), shade);
    }
    p.rect(132, 46, 10, 80, shade);
    p.peak(137, 46, 6, 18, shade);
    p.cv.rect(136, 20, 3, 8, [250, 200, 60]);
    p.cv.disc(137, 18, 2, [250, 200, 60]);
    // The front: a broad gable over five doors and tall pointed windows.
    p.poly(&[(50, 226), (50, 150), (137, 112), (224, 150), (224, 226)], marble);
    for x in [70, 100, 174, 204] {
        p.rect(x - 4, 166, 8, 24, [60, 70, 90]);
        p.peak(x, 166, 4, 6, [60, 70, 90]);
    }
    p.disc(137, 152, 10, [60, 70, 90]);
    p.disc(137, 152, 6, shade);
    for (x, w, h) in [(60, 12, 22), (88, 14, 26), (128, 18, 32), (172, 14, 26), (202, 12, 22)] {
        p.rect(x, 226 - h, w, h, [80, 60, 40]);
        p.peak(x + w / 2, 226 - h, w / 2, 6, [80, 60, 40]);
    }
    // Buttresses running up the front, each ending in a pinnacle.
    for x in [50, 80, 112, 162, 194, 224] {
        let top = 150 - (40 - (x - 137i32).abs() * 40 / 87).max(0);
        p.rect(x - 2, top, 5, 226 - top, shade);
        pinnacle(p, x, top, top - 26, marble);
    }
    // The square, and its pigeons.
    p.ground(226, [190, 180, 170]);
    for _ in 0..30 {
        let (x, y) = (p.rng.below(p.w as usize) as i32, 232 + p.rng.below(46) as i32);
        p.rect(x, y, 3, 2, [110, 110, 120]);
        p.set(x + 3, y, [80, 80, 90]);
    }
    for x in [40, 120, 230] {
        p.person(x, 258 + (x % 9), [40 + x as u8 % 150, 50, 110]);
    }
}

/// A slim Gothic pinnacle rising from `base` to `top`, a statue on its tip.
fn pinnacle(p: &mut Painter, x: i32, base: i32, top: i32, stone: Rgb) {
    p.rect(x - 1, top + 6, 3, base - top - 6, stone);
    p.peak(x, top + 6, 2, 6, stone);
    p.set(x, top - 1, scale(stone, 0.9));
    p.set(x, top - 2, scale(stone, 0.9));
}

fn spain(p: &mut Painter) {
    // Barcelona's Sagrada Família: Gaudí's spindle towers over the Nativity
    // façade, the tall Tower of Jesus Christ behind with its cross, and a
    // building crane, because it has never quite been finished.
    p.sky(196, 40);
    p.clouds(3, 20, 70);
    p.mountains(198, 34, 1.0, [100, 120, 100], 0.55, false);
    p.buildings(0, p.w, 214, (12, 30), &[[220, 200, 170], [200, 170, 140], [230, 220, 200]], 0.45);
    let stone: Rgb = [196, 168, 120];
    let shade = scale(stone, 0.75);
    // Behind: the Tower of Jesus Christ and the Evangelists' towers either
    // side of it.
    spindle(p, 137, 150, 12, 18, shade);
    let cross = if p.after_dark() { [255, 250, 230] } else { [240, 240, 235] };
    p.cv.rect(136, 4, 3, 12, cross);
    p.cv.rect(132, 8, 11, 3, cross);
    for x in [114, 160] {
        spindle(p, x, 156, 9, 46, scale(stone, 0.68));
    }
    // The body of the church, and the façade's three portals under a gable.
    p.rect(52, 152, 170, 66, shade);
    p.rect(60, 160, 154, 58, stone);
    p.peak(137, 162, 34, 34, stone);
    for (x, w, h) in [(80, 16, 26), (127, 20, 34), (178, 16, 26)] {
        p.rect(x, 218 - h, w, h, [70, 55, 45]);
        p.disc(x + w / 2, 218 - h, w / 2, [70, 55, 45]);
    }
    // Gaudí's stonework drips like wax down the gable.
    for _ in 0..70 {
        let x = 96 + p.rng.below(82) as i32;
        let y = 132 + p.rng.below(50) as i32;
        let len = 2 + p.rng.below(5) as i32;
        if (x - 137).abs() < (y - 128) {
            p.rect(x, y, 1, len, scale(stone, 0.85));
        }
    }
    // The Nativity façade's four towers, the middle pair taller.
    for (x, top) in [(72, 64), (102, 42), (172, 42), (202, 64)] {
        spindle(p, x, 168, 9, top, stone);
    }
    // A crane on the right, still at work.
    let yellow: Rgb = [230, 180, 40];
    p.rect(238, 36, 4, 182, yellow);
    for y in (40..216).step_by(8) {
        p.line(238, y, 241, y + 8, scale(yellow, 0.7));
    }
    p.rect(190, 34, 84, 3, yellow);
    p.line(240, 22, 196, 34, [80, 80, 80]);
    p.line(240, 22, 272, 34, [80, 80, 80]);
    p.rect(239, 20, 2, 14, yellow);
    p.rect(252, 37, 12, 7, [110, 110, 110]);
    p.line(212, 37, 212, 60, [60, 60, 60]);
    p.rect(209, 60, 7, 3, [60, 60, 60]);
    // The square in front, with plane trees.
    p.ground(218, [176, 164, 140]);
    p.tree(22, 262, 22, [60, 120, 55]);
    p.tree(256, 266, 20, [60, 120, 55]);
    for x in [96, 150, 190] {
        p.person(x, 252 + (x % 7), [60 + x as u8 % 120, 70, 140]);
    }
}

/// One of the Sagrada Família's towers: a tall tapering spindle standing on
/// `base`, pierced with louvres, topped with a bright mosaic finial at `top`.
fn spindle(p: &mut Painter, x: i32, base: i32, half: i32, top: i32, stone: Rgb) {
    p.poly(&[(x - half, base), (x - half + 2, top + 30), (x - 2, top), (x + 2, top), (x + half - 2, top + 30), (x + half, base)], stone);
    let dark = scale(stone, 0.45);
    for (n, y) in (top + 10..base - 4).step_by(5).enumerate() {
        let spread = 1 + (y - top) * (half - 3) / (base - top).max(1);
        let dx = if n % 2 == 0 { -spread / 2 } else { spread / 2 };
        p.rect(x + dx - 1, y, 2, 2, dark);
    }
    p.rect(x - 1, top - 8, 3, 8, [200, 60, 50]);
    p.disc(x, top - 10, 3, [240, 200, 60]);
    p.set(x - 1, top - 11, [250, 250, 250]);
    p.set(x + 1, top - 9, [250, 250, 250]);
}

fn greece(p: &mut Painter) {
    // The Parthenon on the Acropolis rock above white houses and olive trees.
    p.sky(150, 214);
    p.mountains(158, 24, 1.3, [130, 120, 140], 0.5, false);
    p.poly(&[(14, 206), (36, 152), (60, 140), (214, 138), (238, 150), (260, 206)], [176, 146, 108]);
    for _ in 0..40 {
        let (x, y) = (40 + p.rng.below(200) as i32, 146 + p.rng.below(56) as i32);
        let len = 3 + p.rng.below(6) as i32;
        p.rect(x, y, 1, len, [150, 122, 90]);
    }
    p.rect(66, 132, 142, 7, MARBLE);
    let missing = [9, 10];
    for i in 0..14 {
        if missing.contains(&i) {
            p.rect(74 + i * 9, 124, 5, 8, MARBLE);
            continue;
        }
        p.rect(74 + i * 9, 104, 5, 28, MARBLE);
        p.rect(78 + i * 9, 104, 1, 28, scale(MARBLE, 0.82));
    }
    p.rect(70, 96, 112, 8, MARBLE);
    p.peak(126, 96, 56, 12, MARBLE);
    p.rect(198, 98, 8, 6, MARBLE);
    p.ground(206, [200, 190, 170]);
    for i in 0..9 {
        let (x, w, hgt) = (i * 31 - 4, 24 + (i * 7) % 8, 16 + (i * 11) % 14);
        p.rect(x, 230 - hgt, w, hgt, WHITE);
        p.rect(x + 4, 230 - hgt + 5, 5, 8, [40, 90, 170]);
    }
    for x in [30, 120, 220] {
        p.tree(x, 262, 9, [120, 140, 100]);
    }
}

fn turkiye(p: &mut Painter) {
    // Istanbul's Blue Mosque, its cascading domes and six minarets, above
    // the Bosphorus with a ferry and gulls.
    p.sky(170, 228);
    let lead: Rgb = [122, 128, 145];
    let stone: Rgb = [200, 196, 196];
    p.ellipse(137, 140, 34, 30, lead);
    p.rect(136, 104, 2, 8, [200, 180, 90]);
    for (x, y, rx, ry) in [(100, 158, 20, 15), (174, 158, 20, 15), (78, 170, 11, 8), (196, 170, 11, 8), (137, 160, 22, 12)] {
        p.ellipse(x, y, rx, ry, lead);
    }
    p.rect(104, 140, 66, 16, stone);
    p.rect(58, 172, 158, 34, stone);
    p.windows((64, 180, 150, 16), (10, 10), (4, 6), stone);
    for (x, top) in [(44, 84), (64, 96), (204, 96), (224, 84), (30, 120), (240, 120)] {
        p.rect(x, top, 4, 206 - top, [225, 222, 220]);
        for y in [top + 22, top + 46] {
            p.rect(x - 1, y, 6, 2, stone);
        }
        p.peak(x + 2, top, 3, 16, lead);
    }
    p.water(206, p.h, [50, 90, 150]);
    p.rect(150, 236, 50, 8, WHITE);
    p.rect(150, 243, 50, 3, [180, 40, 40]);
    p.windows((154, 238, 42, 4), (6, 6), (3, 3), WHITE);
    p.rect(172, 228, 6, 8, DARK);
    p.birds(5, 40, 120);
}

fn norway(p: &mut Painter) {
    // A fjord between steep forested cliffs, snow on the peaks beyond, red
    // wooden houses on the shore and a boat on the still water.
    p.sky(128, 210);
    p.mountains(132, 44, 2.0, [110, 120, 140], 0.45, true);
    p.water(150, 248, [40, 80, 110]);
    let cliff: Rgb = [55, 85, 72];
    p.poly(&[(0, 82), (34, 96), (70, 140), (98, 176), (110, 196), (0, 196)], cliff);
    p.poly(&[(274, 74), (238, 92), (200, 140), (176, 176), (166, 196), (274, 196)], cliff);
    p.recolour((0, 70, 274, 130), cliff, scale(cliff, 0.8), |x, y| dither(x, y, 0.35));
    for _ in 0..24 {
        let side = p.rng.below(2) == 0;
        let y = 110 + p.rng.below(80) as i32;
        let x = if side {
            p.rng.below(((y - 90) / 2).max(1) as usize) as i32
        } else {
            274 - p.rng.below(((y - 80) / 2).max(1) as usize) as i32
        };
        p.conifer(x, y, 10, [30, 65, 45]);
    }
    sailboat(p, 140, 214, 18, [190, 40, 40]);
    p.ground(248, [90, 100, 90]);
    for (x, y) in [(18, 254), (62, 258), (212, 256)] {
        p.rect(x, y - 16, 28, 16, [170, 40, 35]);
        p.peak(x + 14, y - 16, 17, 10, [50, 50, 55]);
        p.rect(x, y - 16, 1, 16, WHITE);
        p.rect(x + 27, y - 16, 1, 16, WHITE);
        p.windows((x + 4, y - 12, 22, 6), (8, 8), (4, 4), [170, 40, 35]);
    }
}

fn netherlands(p: &mut Painter) {
    // Amsterdam canal houses with gables, bicycles on the quay, and their
    // reflections in the canal.
    p.sky(110, 200);
    let walls: [Rgb; 5] = [[150, 72, 52], [86, 52, 40], [44, 52, 48], [230, 224, 212], [120, 110, 100]];
    let quay = 204;
    let mut x = -4;
    let mut n = 0;
    while x < p.w {
        let bw = 28 + p.rng.below(8) as i32;
        let tall = 70 + p.rng.below(28) as i32;
        let wall = walls[n % walls.len()];
        let top = quay - tall;
        p.rect(x, top, bw, tall, wall);
        match n % 3 {
            0 => {
                for step in 0..4 {
                    p.rect(x + step * 4, top - step * 5 - 5, bw - step * 8, 5, wall);
                }
            }
            1 => {
                p.rect(x + 6, top - 12, bw - 12, 12, wall);
                p.disc(x + bw / 2, top - 12, bw / 2 - 6, wall);
                p.rect(x + 4, top - 2, bw - 8, 2, WHITE);
            }
            _ => {
                p.peak(x + bw / 2, top, bw / 2, 16, wall);
                p.rect(x + bw / 2 - 1, top - 14, 3, 3, DARK);
            }
        }
        let mut wy = top + 6;
        while wy + 10 < quay - 6 {
            for wx in [x + 5, x + bw / 2 - 2, x + bw - 10] {
                p.rect(wx - 1, wy - 1, 7, 10, WHITE);
                let pane = if p.after_dark() && p.rng.below(2) == 0 { LAMP } else { p.lit([60, 70, 90]) };
                p.cv.rect(wx, wy, 5, 8, pane);
            }
            wy += 14;
        }
        x += bw;
        n += 1;
    }
    p.rect(0, quay, p.w, 10, [120, 115, 110]);
    for bx in [30, 96, 170, 236] {
        for wx in [bx, bx + 10] {
            for (dx, dy) in [(0, -3), (2, -2), (3, 0), (2, 2), (0, 3), (-2, 2), (-3, 0), (-2, -2)] {
                p.set(wx + dx, quay + 4 + dy, DARK);
            }
        }
        p.line(bx, quay + 4, bx + 5, quay - 1, DARK);
        p.line(bx + 5, quay - 1, bx + 10, quay + 4, DARK);
    }
    // Reflections: the houses mirrored and dimmed in the water, with ripples.
    let water = p.lit([50, 80, 90]);
    for y in quay + 10..p.h {
        let src = quay - (y - quay - 10);
        for x in 0..p.w {
            let wobble = ((y * 3 + x) % 7 == 0) as i32;
            let c = p.cv.get(x + wobble, src.max(0)).unwrap_or(water);
            p.cv.set(x, y, mix(c, water, 0.55));
        }
    }
}

fn germany(p: &mut Painter) {
    // The Brandenburg Gate with its Quadriga, the TV tower behind.
    p.sky(170, 60);
    p.rect(214, 40, 6, 150, [205, 205, 210]);
    p.disc(217, 70, 11, [180, 185, 196]);
    p.rect(206, 70, 22, 2, DARK);
    for (i, y) in (10..40).step_by(6).enumerate() {
        p.rect(216, y, 2, 6, if i % 2 == 0 { [220, 40, 40] } else { WHITE });
    }
    for x in (0..p.w).step_by(18) {
        p.tree(x + 9, 206, 8, [60, 115, 60]);
    }
    let sandstone: Rgb = [218, 202, 162];
    for i in 0..6 {
        let x = 70 + i * 26;
        p.rect(x, 152, 9, 60, sandstone);
        p.rect(x + 7, 152, 2, 60, scale(sandstone, 0.8));
    }
    p.rect(64, 138, 146, 14, sandstone);
    p.rect(64, 146, 146, 2, scale(sandstone, 0.8));
    p.rect(96, 124, 82, 14, sandstone);
    p.rect(100, 128, 74, 6, scale(sandstone, 0.85));
    let copper: Rgb = [80, 150, 130];
    p.rect(126, 114, 22, 10, copper);
    for x in [122, 130, 140, 148] {
        p.rect(x, 108, 5, 8, copper);
        p.rect(x + 3, 104, 2, 5, copper);
    }
    p.rect(136, 92, 2, 22, copper);
    p.rect(132, 92, 10, 2, copper);
    p.ground(212, [190, 190, 190]);
    for y in (220..p.h).step_by(10) {
        p.rect(0, y, p.w, 1, [160, 160, 160]);
    }
}

fn ireland(p: &mut Painter) {
    // The Cliffs of Moher dropping into the Atlantic, O'Brien's Tower on
    // top and sheep on the green.
    p.sky(130, 60);
    if !p.after_dark() {
        p.clouds(5, 20, 100);
    }
    p.water(130, p.h, [60, 95, 120]);
    let rock: Rgb = [78, 72, 68];
    // Headlands receding along the coast, fading towards the sky.
    let haze = p.haze();
    for (pts, fade) in
        [([(0, 140), (52, 137), (60, 150), (56, 176), (0, 178)], 0.45), ([(40, 146), (116, 142), (124, 160), (116, 214), (46, 218)], 0.22)]
    {
        let (top, front) = ((pts[0], pts[1]), pts[2].0);
        p.poly(&pts, mix(rock, haze, fade));
        for y in (top.0.1 + 6..pts[3].1).step_by(6) {
            p.rect(top.0.0, y, front - top.0.0, 1, mix(scale(rock, 1.25), haze, fade));
        }
        p.poly(&[top.0, top.1, (top.1.0, top.1.1 + 4), (top.0.0, top.0.1 + 4)], mix(GRASS, haze, fade));
    }
    p.poly(&[(120, 150), (274, 138), (274, 282), (140, 282), (128, 220)], rock);
    for y in (156..282).step_by(7) {
        p.rect(128, y, 146, 1, scale(rock, 1.25));
    }
    p.poly(&[(120, 150), (274, 138), (274, 150), (124, 158)], GRASS);
    p.rect(206, 116, 12, 24, [150, 150, 145]);
    for x in (206..218).step_by(4) {
        p.rect(x, 112, 2, 4, [150, 150, 145]);
    }
    p.rect(210, 124, 3, 5, DARK);
    for y in (150..282).step_by(5) {
        let x = 124 + (y - 150) / 8 + (y % 3);
        for dx in 0..6 {
            if dither(x - dx, y, 0.5) {
                p.set(x - dx, y, WHITE);
            }
        }
    }
    for x in [236, 252] {
        p.ellipse(x, 144, 4, 2, WHITE);
        p.set(x + 4, 143, DARK);
    }
}

fn belgium(p: &mut Painter) {
    // The Atomium: nine steel spheres in the shape of an iron crystal.
    p.sky(170, 220);
    for x in (0..p.w).step_by(16) {
        p.tree(x + 8, 222, 8, [60, 120, 60]);
    }
    let (cx, cy, k) = (137.0f32, 120.0f32, 44.0f32);
    let angle = 0.26f32;
    let mut atoms = vec![(cx, cy - 1.732 * k, 0.0f32), (cx, cy + 1.732 * k, 0.0)];
    for i in 0..3 {
        let a = angle + i as f32 * std::f32::consts::TAU / 3.0;
        atoms.push((cx + 1.633 * k * a.cos(), cy - 0.577 * k + a.sin() * 8.0, a.sin()));
        let b = a + std::f32::consts::PI / 3.0;
        atoms.push((cx + 1.633 * k * b.cos(), cy + 0.577 * k + b.sin() * 8.0, b.sin()));
    }
    // Index 8 is the centre sphere, joined to all eight corners of the cube.
    atoms.push((cx, cy, 0.0));
    let steel: Rgb = [190, 196, 208];
    let at = |i: usize| (atoms[i].0 as i32, atoms[i].1 as i32);
    let mut bonds: Vec<(usize, usize)> = (0..8).map(|i| (8, i)).collect();
    bonds.extend([(0, 2), (0, 4), (0, 6), (1, 3), (1, 5), (1, 7)]);
    bonds.extend([(2, 3), (2, 7), (4, 3), (4, 5), (6, 5), (6, 7)]);
    for (a, b) in bonds {
        let ((x0, y0), (x1, y1)) = (at(a), at(b));
        p.thick_line(x0, y0, x1, y1, 3, scale(steel, 0.8));
    }
    for leg in [3, 5, 7] {
        let (x, y) = at(leg);
        p.thick_line(x, y, x + (x - cx as i32) / 5, 226, 3, scale(steel, 0.7));
    }
    let mut order: Vec<usize> = (0..atoms.len()).collect();
    order.sort_by(|&a, &b| atoms[a].2.total_cmp(&atoms[b].2));
    for i in order {
        let (x, y) = at(i);
        p.disc(x, y, 12, scale(steel, 0.65));
        p.disc(x, y, 11, steel);
        p.disc(x - 3, y - 4, 4, [235, 240, 248]);
        p.rect(x - 10, y + 2, 20, 2, scale(steel, 0.7));
        if p.after_dark() {
            for dx in (-8..9).step_by(4) {
                p.cv.set(x + dx, y + 2, [255, 255, 230]);
            }
        }
    }
    p.ground(226, [80, 140, 70]);
    p.rect(110, 226, 54, p.h - 226, [200, 195, 185]);
}

// ------------------------------------------------------------------ Asia

fn japan(p: &mut Painter) {
    // Mount Fuji behind a five-storey pagoda, framed by cherry blossom.
    p.sky(170, 222);
    let rock = mix([90, 105, 150], p.haze(), 0.25);
    p.poly(&[(0, 182), (110, 80), (152, 78), (274, 182)], rock);
    p.poly(&[(94, 96), (110, 80), (152, 78), (168, 96), (158, 92), (148, 102), (136, 92), (124, 102), (112, 94)], [248, 248, 252]);
    p.rect(0, 178, p.w, 14, mix([200, 210, 230], p.haze(), 0.5));
    p.hills(206, 20, 1.2, [60, 110, 70]);
    let red: Rgb = [196, 54, 44];
    let roof: Rgb = [60, 52, 56];
    let (cx, mut y) = (60, 246);
    for tier in 0..5 {
        let half = 22 - tier * 3;
        p.rect(cx - half + 4, y - 16, (half - 4) * 2, 14, red);
        p.windows((cx - half + 7, y - 13, (half - 7) * 2, 8), (6, 8), (3, 5), red);
        p.rect(cx - half - 4, y - 18, (half + 4) * 2, 3, roof);
        p.rect(cx - half - 6, y - 20, 3, 2, roof);
        p.rect(cx + half + 3, y - 20, 3, 2, roof);
        y -= 18;
    }
    p.rect(cx, y - 22, 1, 22, [200, 170, 80]);
    p.ground(246, [120, 150, 90]);
    for (x, base, r) in [(200, 250, 26), (252, 236, 20), (-6, 282, 22), (150, 286, 16)] {
        p.rect(x - 2, base - r, 4, r, [70, 45, 40]);
        for _ in 0..30 {
            let dx = p.rng.below((r * 2) as usize) as i32 - r;
            let dy = p.rng.below(r as usize) as i32;
            let c = if p.rng.below(3) == 0 { [250, 210, 225] } else { [236, 160, 186] };
            let size = 3 + p.rng.below(3) as i32;
            p.disc(x + dx, base - r - dy, size, c);
        }
    }
}

fn nepal(p: &mut Painter) {
    // Boudhanath stupa: its white dome, the gilded tower with the Buddha's
    // eyes, prayer flags, and the Himalaya behind.
    p.sky(140, 214);
    p.mountains(146, 74, 3.4, [120, 130, 165], 0.35, true);
    p.hills(176, 18, 1.5, [110, 120, 80]);
    p.buildings(0, 274, 208, (14, 34), &[[150, 80, 55], [130, 70, 50], [170, 100, 70]], 0.1);
    let white: Rgb = [244, 244, 238];
    p.rect(52, 214, 170, 12, white);
    p.rect(68, 204, 138, 10, white);
    p.ellipse(137, 204, 54, 34, white);
    p.recolour((137, 168, 60, 40), white, scale(white, 0.86), |x, y| dither(x, y, ((x - 137) as f32 / 60.0).min(1.0)));
    p.rect(52, 204, 170, 2, [200, 60, 50]);
    let gold: Rgb = [228, 190, 100];
    p.rect(121, 154, 32, 18, gold);
    for (ex, flip) in [(128, -1), (146, 1)] {
        p.rect(ex - 3, 160, 7, 1, DARK);
        p.rect(ex - 2, 161, 5, 2, WHITE);
        p.rect(ex, 161, 2, 2, [40, 70, 160]);
        p.set(ex + flip * 3, 159, DARK);
    }
    p.rect(136, 164, 2, 4, DARK);
    p.set(137, 166, DARK);
    for step in 0..9 {
        let y = 154 - step * 4;
        let half = 14 - step;
        p.rect(137 - half, y - 4, half * 2 + 1, 4, gold);
        p.rect(137 - half, y - 1, half * 2 + 1, 1, scale(gold, 0.75));
    }
    p.rect(130, 114, 15, 3, gold);
    p.rect(137, 104, 1, 10, gold);
    let colours: [Rgb; 5] = [[50, 90, 200], [245, 245, 245], [210, 50, 50], [50, 160, 70], [245, 200, 40]];
    for end in [(0, 196), (274, 196), (20, 150), (254, 150)] {
        let (x0, y0) = (137, 112);
        let steps = 22;
        for i in 1..steps {
            let t = i as f32 / steps as f32;
            let x = x0 as f32 + (end.0 - x0) as f32 * t;
            let y = y0 as f32 + (end.1 - y0) as f32 * t + 10.0 * (t * std::f32::consts::PI).sin();
            p.rect(x as i32, y as i32, 3, 4, colours[i % colours.len()]);
        }
    }
    p.ground(226, [170, 160, 150]);
    for x in (0..p.w).step_by(9) {
        p.rect(x, 226, 1, p.h - 226, [150, 140, 130]);
    }
}

fn india(p: &mut Painter) {
    // The Taj Mahal: the onion dome, its minarets, and the long reflecting
    // pool lined with cypresses.
    p.sky(170, 60);
    let marble: Rgb = [240, 236, 228];
    let shade = scale(marble, 0.84);
    p.rect(18, 184, 238, 14, [205, 120, 90]);
    p.rect(18, 184, 238, 3, marble);
    for (x, top, w) in [(26, 104, 9), (240, 104, 9), (60, 118, 7), (208, 118, 7)] {
        p.rect(x, top, w, 184 - top, marble);
        p.rect(x + w - 2, top, 2, 184 - top, shade);
        for y in [top + 24, top + 48] {
            p.rect(x - 1, y, w + 2, 2, shade);
        }
        p.disc(x + w / 2, top - 2, w / 2 + 1, marble);
        p.rect(x + w / 2, top - 10, 1, 6, [200, 170, 90]);
    }
    p.rect(80, 126, 114, 58, marble);
    p.rect(118, 132, 38, 52, shade);
    p.peak(137, 132, 19, 14, shade);
    for x in [86, 100, 160, 174] {
        p.rect(x, 136, 9, 16, shade);
        p.rect(x, 160, 9, 16, shade);
    }
    p.rect(112, 110, 50, 16, marble);
    p.ellipse(137, 98, 30, 26, marble);
    p.recolour((137, 72, 32, 40), marble, shade, |x, y| dither(x, y, ((x - 137) as f32 / 30.0).min(1.0)));
    p.peak(137, 76, 8, 16, marble);
    p.rect(137, 50, 1, 12, [210, 180, 90]);
    for x in [95, 179] {
        p.rect(x - 7, 116, 2, 10, marble);
        p.rect(x + 5, 116, 2, 10, marble);
        p.ellipse(x, 114, 8, 6, marble);
    }
    p.ground(198, [70, 135, 60]);
    // The reflecting pool: a mirror image of the dome, with ripples.
    let water = p.lit([70, 110, 160]);
    for y in 198..p.h {
        let src = 198 - (y - 198) * 3 / 2;
        for x in 117..158 {
            let c = p.cv.get(x + ((y % 5 == 0) as i32), src.max(0)).unwrap_or(water);
            p.cv.set(x, y, mix(c, water, 0.5));
        }
    }
    for (i, x) in [100, 108, 166, 174].into_iter().enumerate() {
        for y in (210..p.h).step_by(24) {
            p.cypress(x + (i as i32 % 2) * 2, y + 20, 22);
        }
    }
}

fn kyrgyzstan(p: &mut Painter) {
    // Yurts on a high summer pasture below the Tian Shan, with horses
    // grazing and Issyk-Kul glinting beyond.
    p.sky(120, 200);
    p.mountains(132, 62, 3.0, [100, 110, 145], 0.35, true);
    p.water(132, 150, [40, 90, 170]);
    p.hills(166, 16, 1.2, [80, 150, 70]);
    for _ in 0..120 {
        let (x, y) = (p.rng.below(p.w as usize) as i32, 170 + p.rng.below(110) as i32);
        let c = [[240, 220, 60], [230, 230, 240], [200, 80, 160]][p.rng.below(3)];
        p.set(x, y, c);
    }
    for (x, base, s) in [(234, 202, 14), (164, 214, 22), (72, 236, 32)] {
        yurt(p, x, base, s);
    }
    horse(p, 140, 268, [120, 75, 45], true);
    horse(p, 30, 256, [215, 210, 200], false);
}

fn yurt(p: &mut Painter, x: i32, base: i32, r: i32) {
    let felt: Rgb = [236, 230, 214];
    p.ellipse(x, base - r * 6 / 10, r, r / 2, felt);
    p.rect(x - r, base - r * 6 / 10, r * 2, r * 6 / 10, felt);
    p.rect(x - r, base - r * 4 / 10, r * 2, (r / 6).max(2), [190, 60, 40]);
    p.rect(x - r / 6, base - r * 9 / 10 - r / 2, r / 3 + 1, (r / 8).max(1), [90, 70, 50]);
    p.rect(x - r / 5, base - r / 2, r * 2 / 5, r / 2, [200, 110, 40]);
    p.recolour((x, base - r * 2, r + 1, r * 2), felt, scale(felt, 0.85), |xx, yy| dither(xx, yy, 0.5));
}

fn horse(p: &mut Painter, x: i32, base: i32, coat: Rgb, left: bool) {
    let d = if left { -1 } else { 1 };
    for dx in [-9, -5, 6, 10] {
        p.rect(x + d * dx, base - 14, 2, 14, scale(coat, 0.85));
    }
    p.ellipse(x, base - 18, 13, 6, coat);
    // Grazing: the neck slopes down to the grass.
    p.thick_line(x + d * 10, base - 20, x + d * 18, base - 6, 4, coat);
    p.ellipse(x + d * 19, base - 4, 3, 2, coat);
    p.thick_line(x + d * 9, base - 23, x + d * 16, base - 10, 1, [50, 35, 25]);
    p.line(x - d * 13, base - 19, x - d * 15, base - 8, [50, 35, 25]);
}

fn yemen(p: &mut Painter) {
    // Old Sana'a: mud-brick tower houses trimmed in white gypsum, with
    // coloured glass in the arched windows that glows after dark.
    p.sky(140, 214);
    p.mountains(160, 52, 1.8, [150, 120, 95], 0.4, false);
    let browns: [Rgb; 3] = [[160, 110, 72], [140, 96, 62], [176, 124, 82]];
    let mut x = -6;
    let mut n = 0;
    while x < p.w {
        let bw = 32 + p.rng.below(14) as i32;
        let tall = 70 + p.rng.below(70) as i32;
        let top = 234 - tall;
        let wall = browns[n % 3];
        p.rect(x, top, bw, tall, wall);
        p.rect(x, top + tall - 24, bw, 24, scale(wall, 0.8));
        p.rect(x, top - 3, bw, 3, WHITE);
        let mut y = top + 6;
        while y + 12 < 234 - 24 {
            p.rect(x, y + 12, bw, 2, WHITE);
            for (i, wx) in (x + 4..x + bw - 8).step_by(10).enumerate() {
                p.rect(wx - 1, y, 8, 10, WHITE);
                p.rect(wx, y + 3, 6, 7, DARK);
                let glass: Rgb = [[220, 60, 60], [250, 210, 60], [60, 120, 220], [60, 170, 90]][(i + n) % 4];
                let c = if p.after_dark() { glass } else { p.lit(scale(glass, 0.8)) };
                p.cv.rect(wx + 1, y, 4, 3, c);
            }
            y += 16;
        }
        x += bw + 2;
        n += 1;
    }
    p.ground(234, [200, 170, 125]);
}

// ---------------------------------------------------------------- Americas

fn united_states(p: &mut Painter) {
    // Miami Beach: pastel Art Deco hotels with neon, palms and a lifeguard hut.
    p.sky(150, 214);
    let pastels: [Rgb; 5] = [[245, 182, 196], [124, 212, 204], [250, 226, 144], [204, 184, 232], [246, 244, 236]];
    let neon: [Rgb; 3] = [[255, 90, 180], [90, 220, 255], [255, 160, 60]];
    let mut x = -4;
    let mut n = 0;
    while x < p.w {
        let bw = 40 + p.rng.below(20) as i32;
        let tall = 52 + p.rng.below(34) as i32;
        let top = 214 - tall;
        let wall = pastels[n % 5];
        p.rect(x, top, bw, tall, wall);
        p.rect(x + bw - 3, top, 3, tall, scale(wall, 0.85));
        let mut y = top + 8;
        while y < 206 {
            p.rect(x + 2, y - 2, bw - 4, 1, scale(wall, 0.6));
            p.windows((x + 5, y, bw - 10, 5), (7, 8), (4, 4), wall);
            y += 11;
        }
        let fin = x + bw / 2;
        p.rect(fin - 2, top - 14, 5, 14, wall);
        let sign = if p.after_dark() { neon[n % 3] } else { p.lit(scale(neon[n % 3], 0.75)) };
        p.cv.rect(fin - 1, top - 12, 3, 10 + tall / 3, sign);
        x += bw + 2;
        n += 1;
    }
    p.rect(0, 214, p.w, 12, [170, 170, 175]);
    for x in [16, 92, 168, 244] {
        p.palm(x, 232, 64);
    }
    p.ground(226, [240, 222, 180]);
    let hut = (196, 262);
    p.rect(hut.0, hut.1 - 30, 4, 30, [240, 240, 240]);
    p.rect(hut.0 + 26, hut.1 - 30, 4, 30, [240, 240, 240]);
    p.rect(hut.0 - 4, hut.1 - 48, 38, 18, [245, 160, 180]);
    p.poly(&[(hut.0 - 8, hut.1 - 48), (hut.0 + 15, hut.1 - 58), (hut.0 + 38, hut.1 - 48)], [80, 190, 210]);
    p.rect(hut.0 + 4, hut.1 - 44, 22, 8, [80, 190, 210]);
}

fn mexico(p: &mut Painter) {
    // The Pyramid of the Sun at Teotihuacan, Popocatépetl smoking beyond,
    // agaves and prickly pears in front.
    p.sky(160, 70);
    let volcano = mix([110, 110, 140], p.haze(), 0.4);
    p.poly(&[(140, 168), (204, 94), (222, 92), (274, 140), (274, 168)], volcano);
    p.poly(&[(190, 108), (204, 94), (222, 92), (238, 106), (226, 104), (214, 110), (202, 104)], [245, 245, 250]);
    for i in 0..6 {
        let (x, y, r): (i32, i32, i32) = (212 - i * 5, 84 - i * 8, 4 + i);
        for yy in y - r..y + r {
            for xx in x - r..x + r {
                if (xx - x).pow(2) + (yy - y).pow(2) < r * r && dither(xx, yy, 0.55) {
                    p.set(xx, yy, [205, 205, 210]);
                }
            }
        }
    }
    p.ground(168, [190, 160, 110]);
    let stone: Rgb = [165, 135, 102];
    for tier in 0..5 {
        let half = 100 - tier * 18;
        let y = 220 - tier * 16;
        p.rect(110 - half, y - 16, half * 2, 16, stone);
        p.rect(110 - half, y - 16, half * 2, 2, scale(stone, 1.15));
        p.rect(110, y - 16, half, 16, scale(stone, 0.85));
    }
    p.rect(100, 140, 20, 80, scale(stone, 1.12));
    for y in (142..220).step_by(3) {
        p.rect(100, y, 20, 1, scale(stone, 0.95));
    }
    for (x, base) in [(236, 262), (30, 268), (190, 274)] {
        for i in -3..=3 {
            p.thick_line(x, base, x + i * 6, base - 20 + i.abs() * 4, 2, [70, 130, 100]);
        }
    }
    for (x, y) in [(258, 240), (264, 228), (252, 230)] {
        p.ellipse(x, y, 5, 7, [80, 140, 70]);
    }
}

fn brazil(p: &mut Painter) {
    // Rio: Christ the Redeemer on Corcovado, Sugarloaf with its cable car,
    // Guanabara Bay and Copacabana's wave-patterned promenade.
    p.sky(140, 216);
    p.poly(&[(0, 164), (0, 104), (26, 76), (42, 62), (54, 64), (74, 92), (112, 164)], [50, 100, 60]);
    p.recolour((0, 60, 112, 104), [50, 100, 60], [80, 90, 95], |x, y| y < 90 && dither(x, y, 0.5));
    let statue = if p.after_dark() { [255, 245, 220] } else { p.lit([235, 235, 230]) };
    p.cv.rect(46, 44, 3, 18, statue);
    p.cv.rect(38, 48, 19, 2, statue);
    p.cv.disc(47, 42, 1, statue);
    p.ellipse(214, 164, 32, 76, [105, 108, 118]);
    p.ellipse(214, 164, 32, 30, [60, 120, 60]);
    p.ellipse(166, 170, 24, 32, [60, 120, 60]);
    p.line(178, 140, 212, 90, DARK);
    p.rect(194, 112, 5, 4, [220, 60, 50]);
    p.water(164, 216, [40, 120, 170]);
    p.ground(216, [240, 222, 180]);
    for y in 246..p.h {
        for x in 0..p.w {
            let v = y as f32 + 3.0 * (x as f32 / 6.0).sin();
            p.set(x, y, if (v / 4.0) as i32 % 2 == 0 { [30, 30, 34] } else { WHITE });
        }
    }
    p.palm(26, 248, 48);
    p.palm(244, 250, 42);
}

fn argentina(p: &mut Painter) {
    // La Boca's Caminito: corrugated-iron houses painted every colour, a
    // cobbled street and a couple dancing the tango.
    p.sky(100, 60);
    let paints: [Rgb; 6] = [[50, 110, 200], [245, 200, 50], [210, 50, 50], [60, 160, 90], [240, 130, 40], [130, 200, 230]];
    let mut x = -6;
    let mut n = 0;
    while x < p.w {
        let bw = 36 + p.rng.below(14) as i32;
        let tall = 70 + p.rng.below(30) as i32;
        let top = 222 - tall;
        let wall = paints[n % 6];
        p.rect(x, top, bw, tall, wall);
        for cx in (x..x + bw).step_by(3) {
            p.rect(cx, top, 1, tall, scale(wall, 0.82));
        }
        let trim = paints[(n + 3) % 6];
        p.rect(x, top, bw, 3, trim);
        p.rect(x + 6, top + 12, 10, 16, trim);
        p.rect(x + bw - 16, top + 12, 10, 16, trim);
        p.rect(x + 2, top + 30, bw - 4, 2, DARK);
        for bx in (x + 3..x + bw - 3).step_by(3) {
            p.rect(bx, top + 26, 1, 4, DARK);
        }
        p.rect(x + bw / 2 - 5, 222 - 22, 10, 22, [90, 60, 40]);
        x += bw;
        n += 1;
    }
    p.ground(222, [130, 120, 115]);
    for y in (224..p.h).step_by(5) {
        let offset = (y / 5 % 2) * 4;
        for x in (offset..p.w).step_by(8) {
            p.rect(x, y, 1, 4, [100, 92, 88]);
        }
        p.rect(0, y, p.w, 1, [100, 92, 88]);
    }
    // Tango dancers.
    let (mx, wx, base) = (124, 140, 272);
    p.line(mx - 3, base, mx, base - 12, DARK);
    p.line(mx + 3, base, mx, base - 12, DARK);
    p.rect(mx - 3, base - 24, 7, 13, DARK);
    p.disc(mx, base - 27, 3, [190, 140, 100]);
    p.rect(mx - 4, base - 31, 9, 2, DARK);
    p.rect(mx - 2, base - 34, 5, 3, DARK);
    p.poly(&[(wx - 7, base - 4), (wx + 6, base - 4), (wx + 1, base - 22), (wx - 2, base - 22)], [210, 30, 40]);
    p.line(wx - 4, base, wx - 2, base - 6, [190, 140, 100]);
    p.line(wx + 6, base - 6, wx + 12, base, [190, 140, 100]);
    p.disc(wx, base - 25, 3, [190, 140, 100]);
    p.disc(wx + 1, base - 27, 3, DARK);
    p.line(mx + 3, base - 21, wx - 2, base - 20, DARK);
}

fn peru(p: &mut Painter) {
    // Machu Picchu: Huayna Picchu's peak over the stone ruins and terraces,
    // wisps of cloud, and a llama.
    p.sky(110, 60);
    p.mountains(140, 60, 2.6, [70, 110, 90], 0.45, false);
    p.poly(&[(116, 196), (146, 84), (156, 74), (170, 82), (210, 196)], [45, 110, 60]);
    p.recolour((116, 74, 94, 122), [45, 110, 60], [80, 95, 85], |x, y| dither(x, y, 0.3));
    if !p.after_dark() {
        p.clouds(4, 110, 140);
    }
    p.hills(210, 24, 1.0, [70, 140, 70]);
    for i in 0..7 {
        let y = 186 + i * 6;
        p.rect(0, y, 120 - i * 10, 2, [150, 145, 140]);
    }
    let wall: Rgb = [160, 154, 146];
    for (x, y, w, hgt) in [(40, 196, 30, 14), (80, 192, 36, 16), (130, 200, 26, 12), (166, 196, 40, 14), (214, 202, 30, 12)] {
        p.rect(x, y, w, hgt, wall);
        for row in (y + 3..y + hgt).step_by(4) {
            p.rect(x, row, w, 1, scale(wall, 0.8));
        }
        p.poly(&[(x + w / 2 - 3, y + hgt), (x + w / 2 + 3, y + hgt), (x + w / 2 + 2, y + 5), (x + w / 2 - 2, y + 5)], DARK);
    }
    p.peak(98, 192, 20, 12, [140, 110, 60]);
    p.grass(214, [80, 145, 70]);
    // A llama.
    let (x, base) = (70, 272);
    let wool: Rgb = [236, 228, 210];
    for dx in [-8, -4, 5, 9] {
        p.rect(x + dx, base - 14, 2, 14, wool);
    }
    p.ellipse(x, base - 18, 12, 6, wool);
    p.rect(x + 8, base - 38, 5, 20, wool);
    p.ellipse(x + 12, base - 39, 5, 3, wool);
    p.rect(x + 9, base - 45, 1, 4, wool);
    p.rect(x + 12, base - 45, 1, 4, wool);
    p.set(x + 13, base - 40, DARK);
    p.rect(x - 2, base - 22, 6, 2, [210, 60, 90]);
}

fn australia(p: &mut Painter) {
    // Sydney: the Opera House's sails on the harbour, the Harbour Bridge's
    // arch and the city behind.
    p.sky(160, 222);
    p.buildings(150, 274, 180, (20, 60), &[[200, 205, 215], [170, 180, 195], [220, 220, 225]], 0.35);
    let steel: Rgb = [110, 115, 128];
    let arch = |x: f32| 184.0 - 58.0 * (1.0 - ((x - 236.0) / 82.0).powi(2));
    for x in 154..p.w {
        let top = arch(x as f32) as i32;
        p.rect(x, top, 1, 3, steel);
        p.rect(x, top + 12, 1, 2, steel);
        if x % 6 == 0 {
            p.line(x, top, x, 184, steel);
        }
    }
    p.rect(140, 184, 134, 5, steel);
    p.rect(146, 160, 14, 30, [190, 180, 160]);
    p.water(184, p.h, [40, 100, 160]);
    p.rect(8, 214, 136, 16, [215, 196, 160]);
    let shell: Rgb = [246, 244, 238];
    for (x, w, hgt) in [(16, 40, 30), (40, 58, 46), (74, 50, 40), (100, 36, 28)] {
        p.poly(&[(x, 214), (x + w / 5, 214 - hgt), (x + w / 3, 214 - hgt + 4), (x + w, 214)], shell);
        p.line(x + w / 5, 214 - hgt, x + w * 2 / 3, 214, scale(shell, 0.8));
        p.line(x + w / 5, 214 - hgt, x + w / 3, 214, scale(shell, 0.8));
    }
    if p.after_dark() {
        for x in (14..140).step_by(5) {
            p.cv.rect(x, 210, 3, 3, LAMP);
        }
    }
    p.rect(180, 236, 34, 6, [40, 110, 70]);
    p.rect(186, 230, 20, 6, [245, 210, 70]);
}

fn venezuela(p: &mut Painter) {
    // Angel Falls: the world's highest waterfall dropping off a tepui into
    // the jungle, with scarlet macaws.
    p.sky(90, 60);
    p.mountains(140, 40, 1.4, [110, 90, 80], 0.5, false);
    let rock: Rgb = [146, 84, 62];
    p.poly(&[(26, 96), (238, 90), (250, 206), (16, 206)], rock);
    for x in (28..246).step_by(4) {
        let len = 20 + p.rng.below(80) as i32;
        let y = 98 + p.rng.below(20) as i32;
        p.rect(x, y, 1, len, scale(rock, 0.78));
    }
    p.rect(26, 90, 212, 8, [60, 120, 60]);
    for y in 96..206 {
        let w = 4 + (y - 96) / 16;
        for x in 118 - w / 2..118 + w / 2 + 1 {
            let c = if dither(x, y + (y % 3), 0.4) { [230, 240, 250] } else { [190, 210, 230] };
            p.set(x, y, c);
        }
    }
    for _ in 0..60 {
        let (x, y) = (100 + p.rng.below(40) as i32, 196 + p.rng.below(20) as i32);
        p.disc(x, y, 2, [235, 240, 245]);
    }
    p.ground(206, [30, 80, 40]);
    for _ in 0..80 {
        let (x, y) = (p.rng.below(p.w as usize) as i32, 214 + p.rng.below(70) as i32);
        let g = [30 + p.rng.below(40) as u8, 100 + p.rng.below(60) as u8, 40 + p.rng.below(20) as u8];
        let r = 6 + p.rng.below(6) as i32;
        p.disc(x, y, r, g);
    }
    for (x, y) in [(60, 150), (72, 160)] {
        p.rect(x, y, 5, 2, [220, 40, 40]);
        p.set(x + 5, y, [250, 210, 60]);
        p.set(x - 1, y + 1, [40, 90, 200]);
    }
}

fn ecuador(p: &mut Painter) {
    // Cotopaxi's perfect snowy cone above Quito's red roofs and the
    // twin towers of the Basílica.
    p.sky(150, 222);
    let cone = mix([80, 85, 110], p.haze(), 0.35);
    p.poly(&[(30, 172), (124, 72), (150, 72), (244, 172)], cone);
    p.poly(&[(100, 98), (124, 72), (150, 72), (174, 98), (162, 94), (150, 102), (137, 92), (124, 102), (112, 94)], [248, 248, 252]);
    p.hills(184, 20, 1.6, [150, 160, 90]);
    let mut x = -4;
    while x < p.w {
        let bw = 18 + p.rng.below(12) as i32;
        let tall = 12 + p.rng.below(12) as i32;
        p.rect(x, 214 - tall, bw, tall, [240, 236, 226]);
        p.peak(x + bw / 2, 214 - tall, bw / 2 + 2, 7, TERRACOTTA);
        p.windows((x + 3, 214 - tall + 3, bw - 6, tall - 6), (6, 7), (2, 3), [240, 236, 226]);
        x += bw + 1;
    }
    let grey: Rgb = [150, 150, 160];
    for tx in [196, 222] {
        p.rect(tx, 122, 14, 92, grey);
        p.peak(tx + 7, 122, 8, 22, grey);
        p.rect(tx + 4, 134, 6, 10, DARK);
    }
    p.rect(196, 160, 40, 54, grey);
    p.disc(216, 176, 8, [180, 60, 60]);
    p.ground(214, [180, 90, 60]);
    for y in (218..p.h).step_by(6) {
        for x in ((y / 6 % 2) * 5..p.w).step_by(10) {
            p.ellipse(x, y, 5, 2, [160, 75, 50]);
        }
    }
}

fn uruguay(p: &mut Painter) {
    // Montevideo across the bay: the Palacio Salvo's tower over the old
    // town, the brown Río de la Plata and the Rambla's sea wall.
    p.sky(150, 60);
    p.mountains(160, 14, 1.0, [100, 110, 100], 0.5, false);
    p.buildings(0, 274, 190, (14, 40), &[[225, 215, 195], [200, 195, 185], [235, 228, 210]], 0.2);
    let cream: Rgb = [226, 210, 178];
    p.rect(128, 116, 46, 74, cream);
    p.windows((132, 122, 38, 64), (6, 8), (3, 4), cream);
    p.rect(138, 72, 26, 44, cream);
    p.windows((142, 78, 18, 34), (6, 8), (3, 4), cream);
    p.rect(144, 54, 14, 18, cream);
    p.disc(151, 54, 7, cream);
    p.rect(150, 30, 2, 24, cream);
    if p.after_dark() {
        p.cv.disc(151, 30, 2, [255, 240, 160]);
    }
    p.water(190, 238, [120, 118, 100]);
    p.rect(0, 238, p.w, 8, [175, 170, 160]);
    p.ground(246, [205, 200, 190]);
    for x in (20..p.w).step_by(60) {
        p.rect(x, 216, 2, 24, DARK);
        p.rect(x - 2, 214, 6, 3, DARK);
        if p.after_dark() {
            p.cv.disc(x + 1, 218, 2, LAMP);
        }
    }
    p.palm(250, 280, 44);
}

// ---------------------------------------------------------- the Caribbean

fn monkey_island(p: &mut Painter) {
    // After the 1990 adventure game: a smoking volcano over the jungle, the
    // giant stone Monkey Head carved into the hillside with its mouth open,
    // a pirate ship at anchor in the bay, and palms on the beach.
    p.sky(150, 214);
    // The volcano behind, with smoke drifting off its crater.
    let rock: Rgb = [92, 80, 78];
    // Far-off colours are already lit, so they go straight onto the canvas.
    let far_rock = p.far(rock, 0.35);
    p.cv.poly(&[(0, 152), (70, 70), (96, 68), (170, 152)], far_rock);
    let smoke: Rgb = [170, 165, 160];
    for (n, (x, y)) in [(84, 60), (92, 50), (102, 42), (114, 36), (128, 32)].into_iter().enumerate() {
        let puff = p.far(smoke, 0.25 + n as f32 * 0.1);
        p.cv.ellipse(x, y, 7 + n as i32 * 2, 5 + n as i32, puff);
    }
    p.water(150, 222, [30, 90, 150]);
    // The jungle hill the head is carved from.
    let jungle: Rgb = [40, 105, 50];
    p.poly(&[(120, 196), (150, 120), (196, 96), (240, 104), (274, 128), (274, 196)], jungle);
    let hill = p.lit(jungle);
    for _ in 0..40 {
        let (x, y, r) = (124 + p.rng.below(150) as i32, 112 + p.rng.below(80) as i32, 3 + p.rng.below(3) as i32);
        // Only on the hill, not floating in the sky or the sea.
        if p.cv.get(x, y) == Some(hill) {
            p.disc(x, y, r, [30, 85, 40]);
        }
    }
    monkey_head(p, 204, 138);
    pirate_ship(p, 58, 182);
    // The beach in front, and palms.
    p.ground(218, [214, 190, 130]);
    for x in (0..p.w).step_by(3) {
        p.set(x, 218, [240, 240, 230]);
    }
    p.palm(28, 266, 60);
    p.palm(250, 262, 54);
}

/// The Monkey Head: a stone ape's face as big as a hill, centred at (cx, cy),
/// its open mouth the way in. At night a fire glows inside.
fn monkey_head(p: &mut Painter, cx: i32, cy: i32) {
    let stone: Rgb = [150, 142, 128];
    let shade: Rgb = [112, 104, 94];
    let dark: Rgb = [30, 24, 22];
    p.ellipse(cx - 34, cy - 4, 9, 13, shade);
    p.ellipse(cx + 34, cy - 4, 9, 13, shade);
    p.ellipse(cx - 34, cy - 4, 5, 8, stone);
    p.ellipse(cx + 34, cy - 4, 5, 8, stone);
    p.ellipse(cx, cy, 30, 38, shade);
    p.ellipse(cx - 3, cy - 2, 26, 35, stone);
    // A heavy brow over deep-set eyes.
    p.rect(cx - 24, cy - 18, 48, 6, shade);
    p.ellipse(cx - 11, cy - 9, 6, 4, dark);
    p.ellipse(cx + 11, cy - 9, 6, 4, dark);
    // The muzzle, with nostrils and the gaping mouth.
    p.ellipse(cx, cy + 14, 19, 15, [168, 160, 146]);
    p.rect(cx - 6, cy + 3, 3, 3, dark);
    p.rect(cx + 3, cy + 3, 3, 3, dark);
    p.ellipse(cx, cy + 19, 11, 8, dark);
    if p.after_dark() {
        // Firelight inside, and eyes catching it: light sources stay bright.
        for (x, y, r, c) in [(cx, cy + 21, 7, [230, 120, 40]), (cx, cy + 22, 4, [255, 200, 90])] {
            p.cv.disc(x, y, r, c);
        }
        p.cv.rect(cx - 12, cy - 10, 2, 2, [255, 220, 120]);
        p.cv.rect(cx + 10, cy - 10, 2, 2, [255, 220, 120]);
    }
}

/// A pirate ship at anchor, its waterline at `base` and its middle at x.
fn pirate_ship(p: &mut Painter, x: i32, base: i32) {
    let hull: Rgb = [60, 38, 28];
    p.poly(&[(x - 34, base - 12), (x + 30, base - 12), (x + 24, base), (x - 28, base)], hull);
    p.rect(x - 34, base - 18, 14, 6, hull);
    p.rect(x - 30, base - 9, 52, 1, [150, 110, 60]);
    for port in 0..5 {
        p.rect(x - 22 + port * 9, base - 7, 3, 2, [20, 14, 10]);
    }
    let sail: Rgb = [230, 222, 200];
    for (mx, h) in [(x - 14, 50), (x + 8, 60)] {
        let top = base - 12 - h;
        let mid = top + h / 2;
        p.rect(mx, top, 2, h, hull);
        // A topsail over a bigger mainsail, both bellied out, with their yards.
        p.poly(&[(mx - 7, top + 6), (mx + 9, top + 6), (mx + 10, mid - 4), (mx - 8, mid - 4)], sail);
        p.poly(&[(mx - 11, mid), (mx + 13, mid), (mx + 15, base - 22), (mx - 13, base - 22)], sail);
        p.rect(mx - 8, top + 6, 18, 1, hull);
        p.rect(mx - 11, mid, 25, 1, hull);
    }
    // The Jolly Roger flying from the main mast.
    p.rect(x + 10, base - 78, 9, 6, [20, 20, 24]);
    p.rect(x + 13, base - 76, 3, 2, [235, 235, 230]);
}
