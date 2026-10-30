//! Every country's postcard at the picture frame's size, two ways side by
//! side: on the left the whole scene shrunk to fit (what the game shows), on
//! the right the full-size scene seen through a window. Run with
//! `cargo run --release --bin postcards`; 4 s each, cycling through the
//! times of day, with paint times in the log.

use std::time::Instant;

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi::config::{Config, DriverConfig};
use esp_idf_svc::hal::spi::{SpiDeviceDriver, SpiDriver};
use esp_idf_svc::hal::units::FromValueType;
use log::info;

use carmen_esp32::art::pixels::Canvas;
use carmen_esp32::art::postcard;
use carmen_esp32::art::scene::TimeOfDay;
use carmen_esp32::display::{Display, H, W};
use carmen_esp32::ui::{self, Fonts, PICTURE, rect};

const COUNTRIES: &str = include_str!("../../countries.json");
const SECONDS_EACH: u32 = 4;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let p = Peripherals::take()?;
    let pins = p.pins;
    let bus = SpiDriver::new(p.spi2, pins.gpio12, pins.gpio11, Some(pins.gpio13), &DriverConfig::new())?;
    let screen = SpiDeviceDriver::new(&bus, Some(pins.gpio10), &Config::new().baudrate(40.MHz().into()))?;
    let mut display = Display::new(screen, PinDriver::output(pins.gpio9)?, PinDriver::output(pins.gpio8)?)?;
    let fonts = Fonts::new();

    let countries: Vec<(String, f64)> = serde_json::from_str::<serde_json::Value>(COUNTRIES)?["countries"]
        .as_array()
        .map(|all| all.iter().map(|c| (c["country"].as_str().unwrap_or("").to_string(), c["lat"].as_f64().unwrap_or(0.0))).collect())
        .unwrap_or_default();

    let times = [TimeOfDay::Day, TimeOfDay::Dusk, TimeOfDay::Night, TimeOfDay::Dawn];
    let (left, right) = (rect(0, 36, 150, 204), rect(166, 36, 150, 204));
    for n in 0.. {
        let (name, lat) = &countries[n % countries.len()];
        let time = times[n / countries.len() % times.len()];
        let (w, h) = (PICTURE.inner().w, PICTURE.inner().h);
        let clock = Instant::now();
        let fitted = postcard::fitted(name, *lat, time, w, h);
        let painted = clock.elapsed();
        let windowed = postcard::windowed(name, *lat, time, w, h);

        let mut cv = Canvas::new(W as i32, H as i32);
        for (frame, card, label) in [(left, &fitted, "fitted (the game)"), (right, &windowed, "windowed")] {
            ui::frame(&mut cv, frame, ui::WHITE);
            let at = frame.inner();
            for (i, &px) in card.pixels().iter().enumerate() {
                cv.set(at.x + i as i32 % w, at.y + i as i32 / w, px);
            }
            fonts.label.draw(&mut cv, frame.x + 2, 14, label, ui::GREY);
        }
        fonts.body.draw(&mut cv, 2, 30, name, ui::YELLOW);
        display.blit(0, 0, W, cv.pixels())?;
        info!("{name} (time of day {}): fitted in {painted:?}", n / countries.len() % times.len());
        FreeRtos::delay_ms(SECONDS_EACH * 1000);
    }
    Ok(())
}
