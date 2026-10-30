//! The touch panel test, run with `cargo run --release --bin touch`.
//!
//! Three crosshairs appear one after another (top left, top right, bottom
//! left): tap the centre of each and let go. That calibrates the panel. Then
//! a white dot follows the finger, leaving a trail; tap the top right corner
//! to clear the screen. The log shows the calibration and each raw reading.

// The test draws no pictures, only rectangles.
#[allow(dead_code)]
#[path = "../display.rs"]
mod display;
#[path = "../touch.rs"]
mod touch;

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi::config::{Config, DriverConfig};
use esp_idf_svc::hal::spi::{SpiDeviceDriver, SpiDriver};
use esp_idf_svc::hal::units::FromValueType;
use log::info;

use display::{Display, H, Rgb, W};
use touch::{Calibration, Touch};

const BLACK: Rgb = [0, 0, 0];
const WHITE: Rgb = [255, 255, 255];
const RED: Rgb = [255, 0, 0];
/// Inset of the targets from the corners: resistive panels read poorly at the very edge.
const INSET: i32 = 24;
const CLEAR_BOX: usize = 40;
/// Raw distance between taps on different targets; they're about 3000 apart.
const MIN_APART: i32 = 500;
/// Readings in a row without a press that count as let go (20 ms each).
const RELEASED: i32 = 15;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let p = Peripherals::take()?;
    let pins = p.pins;
    let bus = SpiDriver::new(p.spi2, pins.gpio12, pins.gpio11, Some(pins.gpio13), &DriverConfig::new())?;
    let screen = SpiDeviceDriver::new(&bus, Some(pins.gpio10), &Config::new().baudrate(40.MHz().into()))?;
    // The touch chip reads at 2 MHz at most.
    let panel = SpiDeviceDriver::new(&bus, Some(pins.gpio7), &Config::new().baudrate(2.MHz().into()))?;
    let mut display = Display::new(screen, PinDriver::output(pins.gpio9)?, PinDriver::output(pins.gpio8)?)?;
    let mut touch = Touch::new(panel, PinDriver::input(pins.gpio6, Pull::Up)?);

    let targets = [(INSET, INSET), (W as i32 - INSET, INSET), (INSET, H as i32 - INSET)];
    let mut raw = [(0, 0); 3];
    for (i, &(x, y)) in targets.iter().enumerate() {
        display.fill(0, 0, W, H, BLACK)?;
        crosshair(&mut display, x as usize, y as usize)?;
        info!("tap the crosshair at ({x}, {y})");
        raw[i] = loop {
            let r = tap(&mut touch)?;
            // A press that lingered, or bounced, counts again where it was.
            if i == 0 || (r.0 - raw[i - 1].0).abs() + (r.1 - raw[i - 1].1).abs() > MIN_APART {
                break r;
            }
            info!("  ignored {r:?}: too close to the last tap");
        };
        info!("  raw reading {:?}", raw[i]);
    }
    let cal = Calibration::from_taps(targets, raw);
    info!("calibration: {cal:?}");
    for (target, r) in targets.iter().zip(raw) {
        info!("  target {target:?}: the stored calibration puts it at {:?}", Calibration::MEASURED.to_screen(r, W, H));
    }

    clear(&mut display)?;
    info!("draw with a finger; tap the red box (top right) to clear");
    loop {
        if let Some(r) = touch.raw()? {
            let (x, y) = cal.to_screen(r, W, H);
            if x >= W - CLEAR_BOX && y < CLEAR_BOX {
                clear(&mut display)?;
                release(&mut touch)?;
                continue;
            }
            info!("raw {r:?} -> screen ({x}, {y})");
            let (x0, y0) = (x.saturating_sub(2).min(W - 5), y.saturating_sub(2).min(H - 5));
            display.fill(x0, y0, 5, 5, WHITE)?;
        }
        FreeRtos::delay_ms(10);
    }
}

fn crosshair(display: &mut Display, x: usize, y: usize) -> anyhow::Result<()> {
    display.fill(x - 10, y, 21, 1, WHITE)?;
    display.fill(x, y - 10, 1, 21, WHITE)?;
    display.fill(x - 1, y - 1, 3, 3, RED)
}

fn clear(display: &mut Display) -> anyhow::Result<()> {
    display.fill(0, 0, W, H, BLACK)?;
    display.fill(W - CLEAR_BOX, 0, CLEAR_BOX, CLEAR_BOX, RED)
}

/// Waits for a press, averages it over a moment, then waits for the release.
fn tap(touch: &mut Touch) -> anyhow::Result<(i32, i32)> {
    let mut readings = Vec::new();
    let mut ticks = 0;
    while readings.len() < 10 {
        if let Some(r) = touch.raw()? {
            readings.push(r);
        } else {
            readings.clear(); // only a steady press counts
        }
        ticks += 1;
        if ticks % 50 == 0 {
            info!("  {}", touch.report()?);
        }
        FreeRtos::delay_ms(20);
    }
    release(touch)?;
    let n = readings.len() as i32;
    Ok((readings.iter().map(|r| r.0).sum::<i32>() / n, readings.iter().map(|r| r.1).sum::<i32>() / n))
}

fn release(touch: &mut Touch) -> anyhow::Result<()> {
    let mut free = 0;
    while free < RELEASED {
        free = if touch.raw()?.is_some() { 0 } else { free + 1 };
        FreeRtos::delay_ms(20);
    }
    Ok(())
}
