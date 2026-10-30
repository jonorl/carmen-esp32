//! Two ways to lay the game out on 320x240, with real content, to choose
//! between on the screen: run with `cargo run --release --bin layout` and
//! tap the screen to go from one to the next (A in English, B in English, A
//! in Spanish, B in Spanish). The log gives each one's text capacity.
//!
//! A, as the original: clock and picture on the left, the text panel on the
//! right with the four buttons under it. B, stacked: picture and clock side
//! by side on top, the text panel across the screen, then the buttons.

use std::time::Instant;

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi::config::{Config, DriverConfig};
use esp_idf_svc::hal::spi::{SpiDeviceDriver, SpiDriver};
use esp_idf_svc::hal::units::FromValueType;
use log::info;

use carmen_esp32::art::pixels::{Canvas, Rgb};
use carmen_esp32::art::scene::{self, TimeOfDay};
use carmen_esp32::display::{Display, H, W};
use carmen_esp32::lang::{self, Lang, tr};
use carmen_esp32::text::{Family, Font};
use carmen_esp32::touch::Touch;

// The game's EGA colours.
const WHITE: Rgb = [255, 255, 255];
const GREY: Rgb = [170, 170, 170];
const YELLOW: Rgb = [255, 255, 85];
const NAVY: Rgb = [0, 0, 170];

/// Lima's latitude, for the postcard.
const LIMA_LAT: f64 = -12.05;

#[derive(Clone, Copy)]
struct Rect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

const fn r(x: i32, y: i32, w: i32, h: i32) -> Rect {
    Rect { x, y, w, h }
}

struct Layout {
    name: &'static str,
    clock: Rect,
    picture: Rect,
    text: Rect,
    buttons: [Rect; 4],
}

const A: Layout = Layout {
    name: "A, as the original",
    clock: r(0, 0, 150, 34),
    picture: r(0, 36, 150, 204),
    text: r(152, 0, 168, 186),
    buttons: [r(152, 188, 42, 52), r(194, 188, 42, 52), r(236, 188, 42, 52), r(278, 188, 42, 52)],
};

const B: Layout = Layout {
    name: "B, stacked",
    clock: r(152, 0, 168, 112),
    picture: r(0, 0, 150, 112),
    text: r(0, 114, 320, 78),
    buttons: [r(0, 194, 80, 46), r(80, 194, 80, 46), r(160, 194, 80, 46), r(240, 194, 80, 46)],
};

/// The frames' double line and gap take this much off each side.
const INSET: i32 = 4;
const PAD: i32 = 3;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let p = Peripherals::take()?;
    let pins = p.pins;
    let bus = SpiDriver::new(p.spi2, pins.gpio12, pins.gpio11, Some(pins.gpio13), &DriverConfig::new())?;
    let screen = SpiDeviceDriver::new(&bus, Some(pins.gpio10), &Config::new().baudrate(40.MHz().into()))?;
    let panel = SpiDeviceDriver::new(&bus, Some(pins.gpio7), &Config::new().baudrate(2.MHz().into()))?;
    let mut display = Display::new(screen, PinDriver::output(pins.gpio9)?, PinDriver::output(pins.gpio8)?)?;
    let mut touch = Touch::new(panel, PinDriver::input(pins.gpio6, Pull::Up)?);

    let font = |family, size| Font::new(family, size).expect("a size built by build.rs");
    let fonts = Fonts { body: font(Family::PixelifySmooth, 13), label: font(Family::Pixelify, 11), heading: font(Family::Pixelify, 22) };

    for n in 0.. {
        let (layout, language) = [(&A, Lang::En), (&B, Lang::En), (&A, Lang::Es), (&B, Lang::Es)][n % 4];
        lang::set(language);
        let clock = Instant::now();
        let cv = draw(layout, &fonts);
        let drawn = clock.elapsed();
        display.blit(0, 0, W, cv.pixels())?;
        info!("{} in {language:?}: drawn in {drawn:?}, sent in {:?}; tap for the next", layout.name, clock.elapsed() - drawn);
        tap(&mut touch)?;
    }
    Ok(())
}

struct Fonts {
    body: Font,
    label: Font,
    heading: Font,
}

fn draw(layout: &Layout, f: &Fonts) -> Canvas {
    let mut cv = Canvas::new(W as i32, H as i32);

    // The clock, and in B, where the room allows, the city as a heading.
    frame(&mut cv, layout.clock);
    let c = inner(layout.clock);
    let mut y = c.y + PAD;
    if layout.clock.h > 60 {
        y = write(&mut cv, &f.heading, c.x + PAD, y, c.w - 2 * PAD, tr("LIMA, PERU", "LIMA, PERÚ"), YELLOW) + 4;
    }
    write(&mut cv, &f.body, c.x + PAD, y, c.w - 2 * PAD, tr("Tuesday, 3 p.m.", "martes, 3 pm"), WHITE);

    // The postcard, painted to fit the picture frame.
    frame(&mut cv, layout.picture);
    let pic = inner(layout.picture);
    let card = scene::paint("Peru", LIMA_LAT, TimeOfDay::Day, pic.w, pic.h);
    for (i, &px) in card.pixels().iter().enumerate() {
        cv.set(pic.x + i as i32 % pic.w, pic.y + i as i32 / pic.w, px);
    }

    // The text panel: where the player is, then what the witness says, paged.
    frame(&mut cv, layout.text);
    let t = inner(layout.text);
    cv.rect(t.x, t.y, t.w, t.h, NAVY);
    let width = t.w - 2 * PAD;
    let mut lines: Vec<(String, Rgb)> =
        f.body.wrap(tr("Tuesday, 3 p.m. at the Hotel", "martes, 3 pm, en el hotel"), width).into_iter().map(|l| (l, GREY)).collect();
    let said = tr(
        "Hotel concierge: “He had brown hair. He wanted to climb the Rwenzori, the snowy 'Mountains of the Moon'.”",
        "Conserje: “Tenía el pelo castaño. Quería escalar los Rwenzori, las nevadas 'Montañas de la Luna'.”",
    );
    lines.extend(f.body.wrap(said, width).into_iter().map(|l| (l, WHITE)));
    let fit = ((t.h - 2 * PAD) / f.body.line_height()) as usize;
    let chars = width / f.body.width("n").max(1);
    info!("  text panel: {fit} lines of about {chars} characters; this statement takes {} lines", lines.len());
    for (k, (line, colour)) in lines.iter().take(fit).enumerate() {
        f.body.draw(&mut cv, t.x + PAD, t.y + PAD + k as i32 * f.body.line_height() + f.body.ascent(), line, *colour);
    }
    if lines.len() > fit {
        // More to come: a tap would turn the page.
        let (ax, ay) = (t.x + t.w - 10, t.y + t.h - 7);
        cv.poly(&[(ax, ay), (ax + 7, ay), (ax + 3, ay + 4)], YELLOW);
    }

    // The four buttons, as the original's icons.
    for (i, &b) in layout.buttons.iter().enumerate() {
        frame(&mut cv, b);
        icon(&mut cv, f, i, inner(b));
    }
    cv
}

/// The original's frames: a double white line.
fn frame(cv: &mut Canvas, b: Rect) {
    for k in [0, 2] {
        let (x, y, w, h) = (b.x + k, b.y + k, b.w - 2 * k, b.h - 2 * k);
        cv.rect(x, y, w, 1, WHITE);
        cv.rect(x, y + h - 1, w, 1, WHITE);
        cv.rect(x, y, 1, h, WHITE);
        cv.rect(x + w - 1, y, 1, h, WHITE);
    }
}

fn inner(b: Rect) -> Rect {
    r(b.x + INSET, b.y + INSET, b.w - 2 * INSET, b.h - 2 * INSET)
}

/// Rough stand-ins for the original's icons: notebook, plane, magnifying glass, computer.
fn icon(cv: &mut Canvas, f: &Fonts, i: usize, b: Rect) {
    let cx = b.x + b.w / 2;
    let label = [tr("NOTES", "NOTAS"), tr("DEPART", "VIAJAR"), "", tr("CRIME", "CRIMEN")][i];
    if !label.is_empty() {
        f.label.draw(cv, cx - f.label.width(label) / 2, b.y + f.label.ascent(), label, WHITE);
    }
    let (top, bottom) = (b.y + f.label.line_height(), b.y + b.h);
    let cy = (top + bottom) / 2;
    match i {
        0 => {
            cv.rect(cx - 9, top + 1, 18, bottom - top - 2, WHITE);
            for ly in (top + 4..bottom - 3).step_by(3) {
                cv.rect(cx - 6, ly, 12, 1, GREY);
            }
        }
        1 => {
            cv.rect(cx - 13, cy - 1, 26, 3, WHITE);
            cv.rect(cx - 2, cy - 8, 4, 16, WHITE);
            cv.rect(cx - 6, cy + 6, 12, 2, WHITE);
        }
        2 => {
            let (lx, ly) = (cx - 2, b.y + b.h / 2 - 3);
            cv.disc(lx, ly, 9, WHITE);
            cv.disc(lx, ly, 6, NAVY);
            cv.line(lx + 6, ly + 6, lx + 12, ly + 12, WHITE);
            cv.line(lx + 7, ly + 6, lx + 13, ly + 12, WHITE);
        }
        _ => {
            cv.rect(cx - 12, top + 1, 24, bottom - top - 8, WHITE);
            cv.rect(cx - 10, top + 3, 20, bottom - top - 12, NAVY);
            cv.rect(cx - 4, bottom - 7, 8, 2, WHITE);
            cv.rect(cx - 9, bottom - 5, 18, 3, WHITE);
        }
    }
}

/// `text` wrapped to `width` from (x, y); returns the y under it.
fn write(cv: &mut Canvas, f: &Font, x: i32, mut y: i32, width: i32, text: &str, colour: Rgb) -> i32 {
    for line in f.wrap(text, width) {
        f.draw(cv, x, y + f.ascent(), &line, colour);
        y += f.line_height();
    }
    y
}

/// Waits for a press, then for the finger to lift.
fn tap(touch: &mut Touch) -> anyhow::Result<()> {
    while touch.raw()?.is_none() {
        FreeRtos::delay_ms(20);
    }
    let mut free = 0;
    while free < 15 {
        free = if touch.raw()?.is_some() { 0 } else { free + 1 };
        FreeRtos::delay_ms(20);
    }
    Ok(())
}
