//! The ILI9341 screen, 320x240 in landscape, through the `mipidsi` driver.
//! Our own driver (set-up commands sent by hand) drew corrupted pictures on
//! this module, while `mipidsi` drew it right with the same wiring (see
//! `src/bin/screen.rs`), so the game draws through it.

use embedded_graphics_core::pixelcolor::Rgb565;
use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::gpio::{Output, PinDriver};
use esp_idf_svc::hal::spi::{SpiDeviceDriver, SpiDriver};
use mipidsi::interface::SpiInterface;
use mipidsi::models::ILI9341Rgb565;
use mipidsi::options::{ColorOrder, Orientation, Rotation};
use mipidsi::{Builder, NoResetPin};

/// The same as the game's `art::pixels::Rgb`, written out so the test
/// programs in `src/bin` can use this file without the art.
pub type Rgb = [u8; 3];

pub const W: usize = 320;
pub const H: usize = 240;

/// Bytes gathered before each SPI write.
const BUFFER: usize = 4096;

type Spi<'d> = SpiDeviceDriver<'d, &'d SpiDriver<'d>>;
type Panel<'d> = mipidsi::Display<SpiInterface<'d, Spi<'d>, PinDriver<'d, Output>>, ILI9341Rgb565, NoResetPin>;

pub struct Display<'d> {
    panel: Panel<'d>,
    // Held high while the panel runs: a dropped driver would let the pin float.
    _reset: PinDriver<'d, Output>,
}

impl<'d> Display<'d> {
    pub fn new(spi: Spi<'d>, dc: PinDriver<'d, Output>, mut reset: PinDriver<'d, Output>) -> anyhow::Result<Self> {
        // Reset by hand rather than through the builder, so the pin stays ours.
        reset.set_low()?;
        Ets::delay_us(10_000);
        reset.set_high()?;
        Ets::delay_us(120_000);
        let buffer = Box::leak(vec![0; BUFFER].into_boxed_slice());
        let panel = Builder::new(ILI9341Rgb565, SpiInterface::new(spi, dc, buffer))
            .orientation(Orientation::new().rotate(Rotation::Deg90))
            .color_order(ColorOrder::Bgr)
            .init(&mut Ets)
            .map_err(|e| anyhow::anyhow!("screen set-up failed: {e:?}"))?;
        Ok(Display { panel, _reset: reset })
    }

    /// Draws `pixels` (`w` wide, row by row) with its top left at (x, y).
    pub fn blit(&mut self, x: usize, y: usize, w: usize, pixels: &[Rgb]) -> anyhow::Result<()> {
        let h = pixels.len() / w;
        self.area(x, y, w, h, pixels.iter().map(|&c| rgb565(c)))
    }

    pub fn fill(&mut self, x: usize, y: usize, w: usize, h: usize, c: Rgb) -> anyhow::Result<()> {
        self.area(x, y, w, h, std::iter::repeat_n(rgb565(c), w * h))
    }

    fn area(&mut self, x: usize, y: usize, w: usize, h: usize, colours: impl IntoIterator<Item = Rgb565>) -> anyhow::Result<()> {
        let (x1, y1) = (x + w - 1, y + h - 1);
        self.panel.set_pixels(x as u16, y as u16, x1 as u16, y1 as u16, colours).map_err(|e| anyhow::anyhow!("screen write failed: {e:?}"))
    }
}

fn rgb565([r, g, b]: Rgb) -> Rgb565 {
    Rgb565::new(r >> 3, g >> 2, b >> 3)
}
