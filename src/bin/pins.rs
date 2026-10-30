//! A wiring check for the screen, run with `cargo run --release --bin pins`
//! and read on the serial monitor. Three parts:
//!
//! 1. Each wired pin on its own: stuck to GND or to a supply, or joined to
//!    another pin (a header sitting in a power rail, two wires in one strip).
//! 2. The display chip answering: needs the display's SDO joined to GPIO 13
//!    for the test. If what it reads back follows what was written, CS, DC,
//!    SDI, SCK and SDO all reach it.
//! 3. Each pin blinking in turn, for a meter or an LED on the display's pins.

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::AnyOutputPin;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi::config::{Config, DriverConfig};
use esp_idf_svc::hal::spi::{SpiDeviceDriver, SpiDriver};
use esp_idf_svc::hal::units::FromValueType;
use esp_idf_svc::sys::*;
use log::info;

const PINS: [(i32, &str); 9] = [
    (4, "piezo"),
    (6, "T_IRQ"),
    (7, "T_CS"),
    (8, "RESET"),
    (9, "DC"),
    (10, "CS"),
    (11, "SDI and T_DIN"),
    (12, "SCK and T_CLK"),
    (13, "T_DO (and SDO for part 2)"),
];
const CS: i32 = 10;
const DC: i32 = 9;
const RESET: i32 = 8;
const T_CS: i32 = 7;
const T_IRQ: i32 = 6;

fn name(pin: i32) -> &'static str {
    PINS.iter().find(|p| p.0 == pin).map_or("?", |p| p.1)
}

fn input(pin: i32, pull: gpio_pull_mode_t) {
    unsafe {
        gpio_reset_pin(pin);
        gpio_set_direction(pin, gpio_mode_t_GPIO_MODE_INPUT);
        gpio_set_pull_mode(pin, pull);
    }
}

fn output(pin: i32, high: bool) {
    unsafe {
        gpio_reset_pin(pin);
        gpio_set_direction(pin, gpio_mode_t_GPIO_MODE_INPUT_OUTPUT);
        gpio_set_level(pin, high as u32);
    }
}

fn level(pin: i32) -> bool {
    unsafe { gpio_get_level(pin) != 0 }
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    FreeRtos::delay_ms(2000); // time for the monitor to attach

    info!("=== Part 1: each pin on its own ===");
    let mut problems = 0;
    for &(pin, what) in &PINS {
        for &(p, _) in &PINS {
            input(p, gpio_pull_mode_t_GPIO_FLOATING);
        }
        input(pin, gpio_pull_mode_t_GPIO_PULLUP_ONLY);
        FreeRtos::delay_ms(5);
        let up = level(pin);
        input(pin, gpio_pull_mode_t_GPIO_PULLDOWN_ONLY);
        FreeRtos::delay_ms(5);
        let down = level(pin);
        match (up, down) {
            (false, _) => {
                info!("GPIO {pin:>2} ({what}): STUCK LOW: touching GND?");
                problems += 1;
            }
            // The touch chip's IRQ line is pulled up on the module, as RESET is on some.
            (true, true) if pin == T_IRQ || pin == RESET => info!("GPIO {pin:>2} ({what}): held high by the module, as expected"),
            (true, true) => {
                info!("GPIO {pin:>2} ({what}): STUCK HIGH: touching 3V3 or 5V?");
                problems += 1;
            }
            (true, false) => info!("GPIO {pin:>2} ({what}): ok"),
        }

        // Joined to another: it follows this pin both ways, against its own pull.
        let mut joined = Vec::new();
        for &(other, _) in PINS.iter().filter(|p| p.0 != pin) {
            output(pin, true);
            input(other, gpio_pull_mode_t_GPIO_PULLDOWN_ONLY);
            FreeRtos::delay_ms(2);
            let follows_high = level(other);
            output(pin, false);
            input(other, gpio_pull_mode_t_GPIO_PULLUP_ONLY);
            FreeRtos::delay_ms(2);
            if follows_high && !level(other) {
                joined.push(other);
            }
        }
        for other in joined {
            info!("    JOINED to GPIO {other} ({}): they share a strip or a wire", name(other));
            problems += 1;
        }
    }
    info!("Part 1: {} problem(s)", problems);

    info!("=== Part 1b: touch the screen and hold for 5 s ===");
    input(T_IRQ, gpio_pull_mode_t_GPIO_FLOATING);
    output(T_CS, true);
    let mut touched = false;
    for _ in 0..50 {
        touched |= !level(T_IRQ);
        FreeRtos::delay_ms(100);
    }
    info!("T_IRQ {}", if touched { "went low: GPIO 6 reaches T_IRQ" } else { "never went low: T_IRQ not reaching GPIO 6, or not pressed" });

    info!("=== Part 2: does the display chip answer? (display SDO joined to GPIO 13) ===");
    let p = Peripherals::take()?;
    let bus = SpiDriver::new(p.spi2, p.pins.gpio12, p.pins.gpio11, Some(p.pins.gpio13), &DriverConfig::new())?;
    // CS by hand, so it stays low from the command through the reply.
    let mut spi = SpiDeviceDriver::new(&bus, None::<AnyOutputPin>, &Config::new().baudrate(1.MHz().into()))?;
    // With nothing answering, read 0xFF rather than noise.
    unsafe { gpio_set_pull_mode(13, gpio_pull_mode_t_GPIO_PULLUP_ONLY) };
    output(CS, true);
    output(DC, true);
    output(T_CS, true);
    output(RESET, false);
    FreeRtos::delay_ms(10);
    unsafe { gpio_set_level(RESET, 1) };
    FreeRtos::delay_ms(150);

    let mut talk = |cmd: u8, data: &[u8], reply: usize| -> anyhow::Result<Vec<u8>> {
        let mut buf = vec![0; reply];
        unsafe {
            gpio_set_level(CS, 0);
            gpio_set_level(DC, 0);
        }
        spi.write(&[cmd])?;
        unsafe { gpio_set_level(DC, 1) };
        if !data.is_empty() {
            spi.write(data)?;
        }
        if reply > 0 {
            spi.read(&mut buf)?;
        }
        unsafe { gpio_set_level(CS, 1) };
        Ok(buf)
    };
    talk(0x11, &[], 0)?; // out of sleep
    FreeRtos::delay_ms(120);
    let id = talk(0xD3, &[], 4)?;
    info!("ID (0xD3): {:02X?}  (an ILI9341 says ?? 00 93 41)", id);
    // Another chip on the same module (an ST7789 says 85 85 52 here) would
    // take the ILI9341's power settings wrongly and wash the colours out.
    info!("ID (0x04): {:02X?}", talk(0x04, &[], 4)?);
    for colmod in [0x55, 0x66] {
        talk(0x3A, &[colmod], 0)?;
        info!("wrote pixel format {colmod:02X}, read back {:02X?}", talk(0x0C, &[], 2)?);
    }
    talk(0x29, &[], 0)?; // display on
    info!("power mode (0x0A): {:02X?}  (awake and on is 9C)", talk(0x0A, &[], 2)?);
    let mut replies = Vec::new();
    for madctl in [0x48, 0x28, 0xE8] {
        talk(0x36, &[madctl], 0)?;
        let r = talk(0x0B, &[], 2)?;
        info!("wrote orientation {madctl:02X}, read back {:02X?}", r);
        replies.push(r);
    }
    let silent = replies.iter().chain([&id]).all(|r| r.iter().all(|&b| b == 0xFF) || r.iter().all(|&b| b == 0));
    let follows = replies[0] != replies[1] && replies[1] != replies[2];
    if follows {
        info!("Part 2: the display answers and follows what it's sent: CS, DC, SDI, SCK, SDO and RESET reach it");
    } else if silent {
        info!("Part 2: NO ANSWER. SDO not on GPIO 13, or CS, SCK, SDI or RESET not reaching the display");
    } else {
        info!("Part 2: it answers but doesn't follow the writes: suspect DC (GPIO 9) or SDI (GPIO 11)");
    }
    drop(spi);
    drop(bus);

    info!("=== Part 3: each pin blinks for 6 s (meter or LED from the display pin to GND) ===");
    loop {
        for &(pin, what) in &PINS {
            info!("now blinking GPIO {pin} -> display {what}");
            for i in 0..12 {
                output(pin, i % 2 == 0);
                FreeRtos::delay_ms(500);
            }
            input(pin, gpio_pull_mode_t_GPIO_FLOATING);
        }
    }
}
