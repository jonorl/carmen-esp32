//! The screen driven by `mipidsi`, a widely used driver, instead of ours: if
//! this shows the test image properly and ours doesn't, the fault is in
//! `display.rs`; if this fails the same way, it's the wiring or the panel.
//! Run with `cargo run --release --bin screen`.
//!
//! Shows mipidsi's test image (a white border, a white triangle in the top
//! left corner, red, green and blue bars labelled R, G and B) for 8 s, then
//! the whole screen red, green, blue and white for 2 s each, and repeats.

use embedded_graphics_core::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics_core::prelude::{DrawTarget, Drawable};
use esp_idf_svc::hal::delay::{Ets, FreeRtos};
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi::config::{Config, DriverConfig};
use esp_idf_svc::hal::spi::{SpiDeviceDriver, SpiDriver};
use esp_idf_svc::hal::units::FromValueType;
use log::info;
use mipidsi::interface::SpiInterface;
use mipidsi::models::ILI9341Rgb565;
use mipidsi::options::{ColorOrder, Orientation, Rotation};
use mipidsi::{Builder, TestImage};

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let p = Peripherals::take()?;
    let pins = p.pins;
    let bus = SpiDriver::new(p.spi2, pins.gpio12, pins.gpio11, Some(pins.gpio13), &DriverConfig::new())?;
    let spi = SpiDeviceDriver::new(&bus, Some(pins.gpio10), &Config::new().baudrate(10.MHz().into()))?;
    let dc = PinDriver::output(pins.gpio9)?;
    let reset = PinDriver::output(pins.gpio8)?;
    let mut buffer = [0u8; 512];
    let interface = SpiInterface::new(spi, dc, &mut buffer);
    let mut display = Builder::new(ILI9341Rgb565, interface)
        .reset_pin(reset)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .color_order(ColorOrder::Bgr)
        .init(&mut Ets)
        .map_err(|e| anyhow::anyhow!("init failed: {e:?}"))?;
    info!("mipidsi set the screen up");

    loop {
        display.clear(Rgb565::BLACK).map_err(|e| anyhow::anyhow!("{e:?}"))?;
        TestImage::new().draw(&mut display).map_err(|e| anyhow::anyhow!("{e:?}"))?;
        info!("test image: white border, white triangle top left, bars R G B");
        FreeRtos::delay_ms(8000);
        for (name, colour) in [("red", Rgb565::RED), ("green", Rgb565::GREEN), ("blue", Rgb565::BLUE), ("white", Rgb565::WHITE)] {
            display.clear(colour).map_err(|e| anyhow::anyhow!("{e:?}"))?;
            info!("whole screen {name}");
            FreeRtos::delay_ms(2000);
        }
    }
}
