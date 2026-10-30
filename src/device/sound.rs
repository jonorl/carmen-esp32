//! The piezo: notes played by the LEDC (PWM) hardware on GPIO 4, a square
//! wave at each note's pitch, from a thread of its own so the game never
//! waits. A new sound cuts off the one playing. The original's tunes are in
//! `tunes.rs`; the effects here stand in for its recorded noises, as clicks
//! in the piezo's loud range (it's faint below 1 kHz).

use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::Duration;

use esp_idf_svc::hal::gpio::OutputPin;
use esp_idf_svc::hal::ledc::config::TimerConfig;
use esp_idf_svc::hal::ledc::{CHANNEL0, LedcDriver, LedcTimerDriver, LowSpeed, Resolution, TIMER0};
use esp_idf_svc::hal::units::FromValueType;
use log::warn;

use crate::device::storage;

/// (pitch in Hz, 0 for a rest; milliseconds).
pub type Notes = &'static [(u16, u16)];

/// A key of the on-screen keyboard.
pub const KEY: Notes = &[(2400, 8)];
/// A page turned, or a button pressed.
pub const TICK: Notes = &[(1800, 5)];
/// One footstep, then the pause before the next: five make the walk to a witness.
pub const WALK: Notes = &[(1200, 14), (0, 360), (1100, 14), (0, 360), (1200, 14), (0, 360), (1100, 14), (0, 360), (1200, 14)];
/// Fourteen footsteps a step every 0.667 s, as the original's walk to jail.
pub const FOOTSTEPS: Notes = &[
    (1150, 14),
    (0, 653),
    (1150, 14),
    (0, 653),
    (1150, 14),
    (0, 653),
    (1150, 14),
    (0, 653),
    (1150, 14),
    (0, 653),
    (1150, 14),
    (0, 653),
    (1150, 14),
    (0, 653),
    (1150, 14),
    (0, 653),
    (1150, 14),
    (0, 653),
    (1150, 14),
    (0, 653),
    (1150, 14),
    (0, 653),
    (1150, 14),
    (0, 653),
    (1150, 14),
    (0, 653),
    (1150, 14),
    (0, 653),
];
/// Taking off: a sweep up from 1.2 to 2.6 kHz and a slower fall to 1.3, 0.8 s.
pub const SHWOOM: Notes = &[
    (1200, 20),
    (1287, 20),
    (1381, 20),
    (1482, 20),
    (1590, 20),
    (1705, 20),
    (1830, 20),
    (1963, 20),
    (2106, 20),
    (2259, 20),
    (2424, 20),
    (2600, 20),
    (2539, 20),
    (2479, 20),
    (2420, 20),
    (2363, 20),
    (2307, 20),
    (2253, 20),
    (2199, 20),
    (2147, 20),
    (2097, 20),
    (2047, 20),
    (1999, 20),
    (1952, 20),
    (1906, 20),
    (1861, 20),
    (1817, 20),
    (1774, 20),
    (1732, 20),
    (1691, 20),
    (1651, 20),
    (1612, 20),
    (1574, 20),
    (1537, 20),
    (1500, 20),
    (1465, 20),
    (1430, 20),
    (1397, 20),
    (1364, 20),
    (1331, 20),
    (1300, 20),
];
/// The prisoner grabbing the bars.
pub const CLANK: Notes = &[(1900, 30), (0, 25), (1500, 50)];

/// Whether sound is on, kept under this name in flash.
const STORE: &str = "sound";

pub struct Sound {
    tx: Sender<Option<Notes>>,
    on: bool,
}

impl Sound {
    pub fn start(timer: TIMER0<'static>, channel0: CHANNEL0<'static>, pin: impl OutputPin + 'static) -> anyhow::Result<Sound> {
        let mut timer = LedcTimerDriver::new(timer, &TimerConfig::new().frequency(1.kHz().into()).resolution(Resolution::Bits8))?;
        let mut pwm = LedcDriver::new(channel0, &timer, pin)?;
        pwm.set_duty(0)?;
        let (tx, rx) = channel();
        std::thread::Builder::new().stack_size(4096).spawn(move || {
            if let Err(e) = play(&mut timer, &mut pwm, rx) {
                warn!("sound stopped: {e}");
            }
        })?;
        Ok(Sound { tx, on: storage::read(STORE).as_deref() != Some("off") })
    }

    /// Plays `notes`, cutting off whatever's playing.
    pub fn play(&self, notes: Notes) {
        if self.on {
            let _ = self.tx.send(Some(notes));
        }
    }

    pub fn stop(&self) {
        let _ = self.tx.send(None);
    }

    pub fn on(&self) -> bool {
        self.on
    }

    /// Turns sound on or off, and remembers it.
    pub fn set(&mut self, on: bool) {
        self.on = on;
        storage::write(STORE, if on { "on" } else { "off" });
        if !on {
            self.stop();
        }
    }
}

fn play(timer: &mut LedcTimerDriver<'static, LowSpeed>, pwm: &mut LedcDriver<'static>, rx: Receiver<Option<Notes>>) -> anyhow::Result<()> {
    let half = pwm.get_max_duty() / 2;
    let mut next = None;
    loop {
        let notes = match next.take() {
            Some(n) => n,
            None => match rx.recv() {
                Ok(n) => n,
                Err(_) => return Ok(()), // the game's gone
            },
        };
        let Some(notes) = notes else { continue };
        for &(hz, ms) in notes {
            let hz = fold(hz as u32);
            if hz == 0 || timer.set_frequency(hz.Hz()).is_err() {
                pwm.set_duty(0)?;
            } else {
                pwm.set_duty(half)?;
            }
            match rx.recv_timeout(Duration::from_millis(ms as u64)) {
                Ok(n) => {
                    next = Some(n);
                    break;
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return Ok(()),
            }
        }
        pwm.set_duty(0)?;
    }
}

/// The same note moved by whole octaves into `PLAYABLE`: the timer can't go
/// much below 300 Hz at this resolution, and the piezo is shrill above 5 kHz.
/// The transcriptions now and then catch a stray octave either way.
fn fold(mut hz: u32) -> u32 {
    if hz == 0 {
        return 0;
    }
    while hz < PLAYABLE.0 {
        hz *= 2;
    }
    while hz > PLAYABLE.1 {
        hz /= 2;
    }
    hz
}

const PLAYABLE: (u32, u32) = (500, 5000);
