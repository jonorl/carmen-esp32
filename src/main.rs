//! Where in the World? on an ESP32-S3 with a 320x240 ILI9341 touch screen:
//! sets up the screen, the touch panel and flash, loads the data, and hands
//! each touch to the game (`app.rs`), redrawing the screen after each.

use std::rc::Rc;
use std::time::Instant;

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi::config::{Config, DriverConfig};
use esp_idf_svc::hal::spi::{SpiDeviceDriver, SpiDriver};
use esp_idf_svc::hal::units::FromValueType;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use log::info;

use carmen_esp32::app::App;
use carmen_esp32::device::storage;
use carmen_esp32::display::{Display, H, W};
use carmen_esp32::lang;
use carmen_esp32::rules::data;
use carmen_esp32::touch::{Calibration, Touch};

/// How often the touch panel is read.
const POLL_MS: u32 = 15;
/// Readings in a row without a press that count as the finger lifting:
/// fewer and one press reads as two.
const RELEASED: u32 = 8;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // Wiring: see README.md.
    let p = Peripherals::take()?;
    let pins = p.pins;
    let bus = SpiDriver::new(p.spi2, pins.gpio12, pins.gpio11, Some(pins.gpio13), &DriverConfig::new())?;
    // The panel's own limit is about 40 MHz; drop this if pictures corrupt on long wires.
    let screen = SpiDeviceDriver::new(&bus, Some(pins.gpio10), &Config::new().baudrate(40.MHz().into()))?;
    // The touch chip reads at 2 MHz at most.
    let panel = SpiDeviceDriver::new(&bus, Some(pins.gpio7), &Config::new().baudrate(2.MHz().into()))?;
    let mut display = Display::new(screen, PinDriver::output(pins.gpio9)?, PinDriver::output(pins.gpio8)?)?;
    let mut touch = Touch::new(panel, PinDriver::input(pins.gpio6, Pull::Up)?);
    let calibration = Calibration::MEASURED;

    storage::start(EspDefaultNvsPartition::take()?)?;
    lang::start();
    let data = match data::load(None) {
        Ok((data, _)) => Rc::new(data),
        Err(data::LoadError::Read(e)) => anyhow::bail!("can't read the data files: {e}"),
        Err(data::LoadError::Invalid(errors)) => anyhow::bail!("the data files have errors:\n{}", errors.join("\n")),
    };
    let mut app = App::new(data, || unsafe { esp_idf_svc::sys::esp_random() } as u64);
    show(&mut display, &mut app)?;

    let mut down = false;
    let mut free = 0;
    loop {
        match touch.raw()? {
            Some(raw) => {
                free = 0;
                if !down {
                    down = true;
                    let (x, y) = calibration.to_screen(raw, W, H);
                    if app.press(x as i32, y as i32) {
                        show(&mut display, &mut app)?;
                    }
                }
            }
            None if down => {
                free += 1;
                if free >= RELEASED {
                    down = false;
                    if app.release() {
                        show(&mut display, &mut app)?;
                    }
                }
            }
            None => {}
        }
        FreeRtos::delay_ms(POLL_MS);
    }
}

fn show(display: &mut Display, app: &mut App) -> anyhow::Result<()> {
    let clock = Instant::now();
    let cv = app.draw();
    let drawn = clock.elapsed();
    display.blit(0, 0, W, cv.pixels())?;
    info!("screen drawn in {drawn:?}, sent in {:?}", clock.elapsed() - drawn);
    Ok(())
}
