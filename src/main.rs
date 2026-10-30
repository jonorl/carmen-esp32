//! Where in the World? on an ESP32-S3 with a 320x240 ILI9341 touch screen.
//! For now: a test pattern, then every country's postcard in turn, to try
//! the screen and see how fast the device paints.

use std::time::Instant;

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi::config::{Config, DriverConfig};
use esp_idf_svc::hal::spi::{SpiDeviceDriver, SpiDriver};
use esp_idf_svc::hal::units::FromValueType;
use log::info;

use carmen_esp32::art::scene::{self, TimeOfDay};
use carmen_esp32::display::{Display, H, W};

const COUNTRIES: &str = include_str!("../countries.json");
const SECONDS_EACH: u32 = 4;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // Wiring: see esp32/README.md.
    let p = Peripherals::take()?;
    let pins = p.pins;
    let bus = SpiDriver::new(
        p.spi2,
        pins.gpio12,       // SCK
        pins.gpio11,       // MOSI
        Some(pins.gpio13), // MISO, from the touch chip only
        // Without DMA, as in `bin/screen.rs`, the set-up known to work.
        &DriverConfig::new(),
    )?;
    // The panel's own limit is about 40 MHz; drop this if pictures corrupt on long wires.
    let screen = SpiDeviceDriver::new(&bus, Some(pins.gpio10), &Config::new().baudrate(40.MHz().into()))?;
    let dc = PinDriver::output(pins.gpio9)?;
    let reset = PinDriver::output(pins.gpio8)?;
    let mut display = Display::new(screen, dc, reset)?;

    info!("screen set up; test pattern for 3 s");
    test_pattern(&mut display)?;
    FreeRtos::delay_ms(3000);

    let countries: Vec<(String, f64)> = serde_json::from_str::<serde_json::Value>(COUNTRIES)?["countries"]
        .as_array()
        .map(|all| all.iter().map(|c| (c["country"].as_str().unwrap_or("").to_string(), c["lat"].as_f64().unwrap_or(0.0))).collect())
        .unwrap_or_default();
    info!("{} countries", countries.len());

    let times = [TimeOfDay::Day, TimeOfDay::Dusk, TimeOfDay::Night, TimeOfDay::Dawn];
    for n in 0.. {
        let (name, lat) = &countries[n % countries.len()];
        let time = times[n / countries.len() % times.len()];
        let start = Instant::now();
        let card = scene::paint(name, *lat, time, W as i32, H as i32);
        let painted = start.elapsed();
        display.blit(0, 0, W, card.pixels())?;
        info!("{name}: painted in {painted:?}, sent in {:?}", start.elapsed() - painted);
        FreeRtos::delay_ms(SECONDS_EACH * 1000);
    }
    Ok(())
}

/// Colour bars with a white border and a red corner at the top left, to
/// check the orientation, the colour order and that the edges line up.
fn test_pattern(display: &mut Display) -> anyhow::Result<()> {
    let bars = [[255, 255, 255], [255, 255, 0], [0, 255, 255], [0, 255, 0], [255, 0, 255], [255, 0, 0], [0, 0, 255], [0, 0, 0]];
    let bar_w = W / bars.len();
    for (i, &c) in bars.iter().enumerate() {
        display.fill(i * bar_w, 0, bar_w, H, c)?;
    }
    display.fill(0, 0, W, 1, [255, 255, 255])?;
    display.fill(0, H - 1, W, 1, [255, 255, 255])?;
    display.fill(0, 0, 1, H, [255, 255, 255])?;
    display.fill(W - 1, 0, 1, H, [255, 255, 255])?;
    display.fill(4, 4, 24, 24, [255, 0, 0])?;
    Ok(())
}
