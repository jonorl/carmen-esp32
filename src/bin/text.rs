//! The fonts on the screen: run with `cargo run --release --bin text`.
//! Pages of the game's own text in English and Spanish, 10 s each, round
//! and round, named in the log: body text (Pixelify 13 px smoothed) across
//! the screen and in a half-screen panel, labels at 11 px, headings at
//! 22 px, and the printer's VT323 at 25 px.

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi::config::{Config, DriverConfig};
use esp_idf_svc::hal::spi::{SpiDeviceDriver, SpiDriver};
use esp_idf_svc::hal::units::FromValueType;
use log::info;

use carmen_esp32::art::pixels::{Canvas, Rgb};
use carmen_esp32::display::{Display, H, W};
use carmen_esp32::text::{Family, Font};

const WHITE: Rgb = [255, 255, 255];
const GREY: Rgb = [150, 150, 150];
const YELLOW: Rgb = [255, 220, 90];
const PAPER: Rgb = [235, 230, 210];
const INK: Rgb = [30, 30, 40];
const BLUE: Rgb = [0, 0, 170];
const MARGIN: i32 = 6;

const CLUE_EN: &str = "Hotel concierge: “He had brown hair. He wanted to climb the Rwenzori, the snowy 'Mountains of the Moon'.”";
const CLUE_ES: &str =
    "Boletera: “Preguntó cómo dormir en una yurta junto al lago Song-Kul. Dijo que no hay nada como moverse en monopatín.”";
const FLASH_ES: &str = "¡URGENTE! Robo en Port Louis, Mauricio: ¡el último huevo de dodo!";
const BRIEF_ES: &str = "Tu misión, Detective Jon: seguí el rastro, conseguí una orden de arresto en la computadora y hacé el arresto antes del próximo martes, 9 am.";
const CHARS: &str = "ÁÉÍÓÚÜÑ áéíóúüñ ¿¡ “” ‘’ – — … € ₹ 0123456789";
const PRINTER: &str = "Volás de Port Louis a Lima (18 horas).";

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let p = Peripherals::take()?;
    let pins = p.pins;
    let bus = SpiDriver::new(p.spi2, pins.gpio12, pins.gpio11, Some(pins.gpio13), &DriverConfig::new())?;
    let screen = SpiDeviceDriver::new(&bus, Some(pins.gpio10), &Config::new().baudrate(40.MHz().into()))?;
    let mut display = Display::new(screen, PinDriver::output(pins.gpio9)?, PinDriver::output(pins.gpio8)?)?;

    let font = |family, size| Font::new(family, size).expect("a size built by build.rs");
    let (small, big, mono) = (font(Family::Pixelify, 11), font(Family::Pixelify, 22), font(Family::Vt323, 25));
    let body = font(Family::PixelifySmooth, 13);
    let full = W as i32 - 2 * MARGIN;
    let pages: [(&str, Box<dyn Fn(&mut Canvas)>); 5] = [
        ("1: body text, Pixelify 13 px smoothed", Box::new(|cv| sample(cv, &body))),
        ("2: body text in a half-screen panel", Box::new(|cv| panel(cv, &body))),
        ("3: labels, Pixelify 11 px (its own pixels)", Box::new(|cv| sample(cv, &small))),
        (
            "4: headings, Pixelify 22 px (each pixel doubled), over body text",
            Box::new(|cv| {
                let mut y = MARGIN;
                y = paragraph(cv, &big, MARGIN, full, y, "LIMA, PERÚ", YELLOW);
                y = paragraph(cv, &big, MARGIN, full, y, "¡CASO CERRADO!", WHITE) + 4;
                y = paragraph(cv, &body, MARGIN, full, y, "martes, mediodía", GREY) + 2;
                paragraph(cv, &body, MARGIN, full, y, CLUE_ES, WHITE);
            }),
        ),
        (
            "5: VT323 25 px, the printer's font",
            Box::new(|cv| {
                cv.rect(0, 0, W as i32, H as i32, PAPER);
                let columns = full / mono.width("M").max(1);
                let mut y = paragraph(cv, &small, MARGIN, full, MARGIN, &format!("{columns} columns across the screen"), INK);
                y = paragraph(cv, &mono, MARGIN, full, y + 2, PRINTER, INK);
                paragraph(cv, &mono, MARGIN, full, y, "¿Qué? ñ ₹ 1234", INK);
            }),
        ),
    ];

    loop {
        for (title, draw) in &pages {
            let mut cv = Canvas::new(W as i32, H as i32);
            draw(&mut cv);
            display.blit(0, 0, W, cv.pixels())?;
            info!("page {title}");
            FreeRtos::delay_ms(10_000);
        }
    }
}

/// The same text at any size: English and Spanish, on black and on paper.
fn sample(cv: &mut Canvas, f: &Font) {
    let full = W as i32 - 2 * MARGIN;
    let mut y = MARGIN;
    y = paragraph(cv, f, MARGIN, full, y, CLUE_EN, WHITE) + 4;
    y = paragraph(cv, f, MARGIN, full, y, FLASH_ES, YELLOW) + 4;
    y = paragraph(cv, f, MARGIN, full, y, BRIEF_ES, WHITE) + 4;
    cv.rect(0, y, W as i32, H as i32 - y, PAPER);
    y = paragraph(cv, f, MARGIN, full, y + 4, CLUE_ES, INK) + 4;
    paragraph(cv, f, MARGIN, full, y, CHARS, INK);
}

/// A text panel half the screen wide, as the game's would be, with how much fits.
fn panel(cv: &mut Canvas, f: &Font) {
    let half = W as i32 / 2;
    cv.rect(half, 0, half, H as i32, BLUE);
    let mut y = MARGIN;
    for text in [FLASH_ES, CLUE_EN, CLUE_ES] {
        y = paragraph(cv, f, half + MARGIN, half - 2 * MARGIN, y, text, WHITE) + 4;
    }
    let lines = f.wrap(BRIEF_ES, half - 2 * MARGIN).len();
    let fit = (H as i32 - 2 * MARGIN) / f.line_height();
    let note = format!("{} px: {fit} lines fit down the screen; the briefing takes {lines} at this width", f.size());
    paragraph(cv, f, MARGIN, half - 2 * MARGIN, MARGIN, &note, GREY);
}

/// `text` wrapped to `width` from (x, y); returns the y under it.
/// Lines that would run off the bottom aren't drawn: a red mark at the
/// bottom right shows the cut, and the log says how many were left out.
fn paragraph(cv: &mut Canvas, f: &Font, x: i32, width: i32, mut y: i32, text: &str, colour: Rgb) -> i32 {
    let lines = f.wrap(text, width);
    for (n, line) in lines.iter().enumerate() {
        if y + f.line_height() > H as i32 {
            info!(
                "  {} px: {} of {} lines of \"{}…\" don't fit",
                f.size(),
                lines.len() - n,
                lines.len(),
                &text.chars().take(20).collect::<String>()
            );
            cv.rect(W as i32 - 12, H as i32 - 8, 8, 6, [255, 80, 80]);
            return H as i32;
        }
        f.draw(cv, x, y + f.ascent(), line, colour);
        y += f.line_height();
    }
    y
}
