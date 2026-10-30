//! The piezo, on its own: run with `cargo run --release --bin sound`. Plays
//! a rising scale, then every effect and every tune in turn, naming each in
//! the log, round and round.

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use log::info;

use carmen_esp32::device::sound::{self, Notes, Sound};
use carmen_esp32::device::{storage, tunes};

/// C major from C5 to C8, to hear where the piezo is loudest.
const SCALE: Notes = &[
    (523, 250),
    (587, 250),
    (659, 250),
    (698, 250),
    (784, 250),
    (880, 250),
    (988, 250),
    (1047, 250),
    (1175, 250),
    (1319, 250),
    (1397, 250),
    (1568, 250),
    (1760, 250),
    (1976, 250),
    (2093, 250),
    (2349, 250),
    (2637, 250),
    (2794, 250),
    (3136, 250),
    (3520, 250),
    (3951, 250),
    (4186, 400),
];

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let p = Peripherals::take()?;
    storage::start(EspDefaultNvsPartition::take()?)?;
    let mut sound = Sound::start(p.ledc.timer0, p.ledc.channel0, p.pins.gpio4)?;
    // A check should be heard whatever the game was last set to.
    sound.set(true);

    let all: [(&str, Notes); 18] = [
        ("scale, C5 to C8", SCALE),
        ("key", sound::KEY),
        ("tick", sound::TICK),
        ("walk to a witness", sound::WALK),
        ("clank", sound::CLANK),
        ("henchman striding", tunes::HENCHMAN_STRIDE),
        ("henchman sneaking", tunes::HENCHMAN_SNEAK),
        ("henchman popping up", tunes::HENCHMAN_POPUP),
        ("axe", tunes::DANGER_AXE),
        ("knife", tunes::DANGER_KNIFE),
        ("pistol", tunes::DANGER_GUN),
        ("crime computer", tunes::COMPUTING),
        ("take-off", sound::SHWOOM),
        ("flight tune (not used)", tunes::TRAVELLING),
        ("chase", tunes::CHASE),
        ("captured", tunes::CAPTURED),
        ("got away", tunes::DISAPPOINT),
        ("walk to jail", sound::FOOTSTEPS),
    ];
    loop {
        for (name, notes) in all {
            let ms: u32 = notes.iter().map(|&(_, ms)| ms as u32).sum();
            info!("{name} ({:.1} s)", ms as f32 / 1000.0);
            sound.play(notes);
            FreeRtos::delay_ms(ms + 1000);
        }
    }
}
