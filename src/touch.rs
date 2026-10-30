//! The XPT2046 resistive touch panel, on the screen's SPI bus with its own
//! chip select. It reports raw 12-bit readings along the panel's own axes,
//! which `Calibration` turns into screen pixels: which raw axis runs along
//! the screen's width, and how raw values map to pixels, both found by
//! tapping three targets (see `src/bin/touch.rs`).

use esp_idf_svc::hal::gpio::{Input, PinDriver};
use esp_idf_svc::hal::spi::{SpiDeviceDriver, SpiDriver};

// Control bytes: start bit, channel, 12-bit, differential, and power down
// between readings with the pen interrupt left on.
const READ_X: u8 = 0xD0;
const READ_Y: u8 = 0x90;
const READ_Z1: u8 = 0xB0;
const READ_Z2: u8 = 0xC0;

/// Readings taken per touch; the median throws out the noisy ones.
const SAMPLES: usize = 7;
/// Below this the press is too light to read reliably.
const MIN_PRESSURE: i32 = 200;

type Spi<'d> = SpiDeviceDriver<'d, &'d SpiDriver<'d>>;

pub struct Touch<'d> {
    spi: Spi<'d>,
    /// Pulled low by the panel while pressed.
    irq: PinDriver<'d, Input>,
}

impl<'d> Touch<'d> {
    pub fn new(spi: Spi<'d>, irq: PinDriver<'d, Input>) -> Self {
        Touch { spi, irq }
    }

    fn read(&mut self, cmd: u8) -> anyhow::Result<i32> {
        let mut rx = [0; 3];
        self.spi.transfer(&mut rx, &[cmd, 0, 0])?;
        Ok((((rx[1] as i32) << 8) | rx[2] as i32) >> 3)
    }

    /// How hard the panel is pressed: about 0 when untouched.
    fn pressure(&mut self) -> anyhow::Result<i32> {
        Ok(self.read(READ_Z1)? + 4095 - self.read(READ_Z2)?)
    }

    /// The raw position while pressed firmly enough, or None. Decided by the
    /// pressure, not T_IRQ, which stayed high through presses on this module.
    pub fn raw(&mut self) -> anyhow::Result<Option<(i32, i32)>> {
        if self.pressure()? < MIN_PRESSURE {
            return Ok(None);
        }
        let (mut xs, mut ys) = ([0; SAMPLES], [0; SAMPLES]);
        for i in 0..SAMPLES {
            xs[i] = self.read(READ_X)?;
            ys[i] = self.read(READ_Y)?;
        }
        // Let go while sampling: the readings are half air.
        if self.pressure()? < MIN_PRESSURE {
            return Ok(None);
        }
        xs.sort_unstable();
        ys.sort_unstable();
        Ok(Some((xs[SAMPLES / 2], ys[SAMPLES / 2])))
    }

    /// One line of raw readings, for working out what the panel is doing.
    pub fn report(&mut self) -> anyhow::Result<String> {
        let irq = if self.irq.is_low() { "low" } else { "high" };
        let (x, y, z1, z2) = (self.read(READ_X)?, self.read(READ_Y)?, self.read(READ_Z1)?, self.read(READ_Z2)?);
        Ok(format!("T_IRQ {irq}, x {x}, y {y}, z1 {z1}, z2 {z2}, pressure {}", z1 + 4095 - z2))
    }
}

/// Raw readings to screen pixels.
#[derive(Clone, Copy, Debug)]
pub struct Calibration {
    /// The raw X axis runs along the screen's height rather than its width.
    pub swap: bool,
    /// Screen x = raw * ax + bx, screen y = raw * ay + by (after any swap).
    pub ax: f32,
    pub bx: f32,
    pub ay: f32,
    pub by: f32,
}

impl Calibration {
    /// Measured on the first module with `src/bin/touch.rs`: its panel is
    /// turned 90 degrees to the screen, so raw X runs down the screen. Another
    /// module of the same kind should be close; rerun the test if taps land off.
    pub const MEASURED: Calibration = Calibration { swap: true, ax: 0.08885985, bx: -20.518784, ay: 0.06678261, by: -20.143303 };

    /// From the raw readings at three screen points: `a`, `b` straight to the
    /// right of `a`, and `c` straight below it.
    pub fn from_taps(screen: [(i32, i32); 3], raw: [(i32, i32); 3]) -> Calibration {
        let [(sa, ra), (sb, rb), (sc, rc)] = [(screen[0], raw[0]), (screen[1], raw[1]), (screen[2], raw[2])];
        // Moving right on the screen changes one raw axis much more than the other.
        let swap = (rb.1 - ra.1).abs() > (rb.0 - ra.0).abs();
        let pick = |r: (i32, i32)| if swap { (r.1, r.0) } else { r };
        let (pa, pb, pc) = (pick(ra), pick(rb), pick(rc));
        let ax = (sb.0 - sa.0) as f32 / (pb.0 - pa.0) as f32;
        let ay = (sc.1 - sa.1) as f32 / (pc.1 - pa.1) as f32;
        Calibration { swap, ax, bx: sa.0 as f32 - pa.0 as f32 * ax, ay, by: sa.1 as f32 - pa.1 as f32 * ay }
    }

    pub fn to_screen(&self, raw: (i32, i32), w: usize, h: usize) -> (usize, usize) {
        let (rx, ry) = if self.swap { (raw.1, raw.0) } else { raw };
        let x = (rx as f32 * self.ax + self.bx).clamp(0.0, (w - 1) as f32);
        let y = (ry as f32 * self.ay + self.by).clamp(0.0, (h - 1) as f32);
        (x as usize, y as usize)
    }
}
