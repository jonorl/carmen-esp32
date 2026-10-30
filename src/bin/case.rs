//! The game's rules running on the device, with nothing drawn: run with
//! `cargo run --release --bin case` and read the serial monitor. It loads and
//! checks the data, then plays one case in English and one in Spanish as a
//! careful player would (every witness, the right flights, the full warrant),
//! printing everything the game says, with timings and free memory. It also
//! counts its starts in flash, to show saving survives a reset.

use std::rc::Rc;
use std::time::Instant;

use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys::{esp_get_free_heap_size, esp_random};
use log::info;

use carmen_esp32::device::storage;
use carmen_esp32::lang::{self, Lang};
use carmen_esp32::rules::data;
use carmen_esp32::rules::game::{Game, RANKS, Tone, clock_text};
use carmen_esp32::rules::rng::Rng;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    storage::start(EspDefaultNvsPartition::take()?)?;
    let starts = storage::read("starts").and_then(|s| s.parse::<u32>().ok()).unwrap_or(0) + 1;
    storage::write("starts", &starts.to_string());
    info!("start number {starts} (kept in flash: reset the board and it goes up)");
    lang::start();
    info!("language at start: {:?}", lang::lang());

    let free = heap();
    let clock = Instant::now();
    let data = match data::load(None) {
        Ok((data, warnings)) => {
            for w in &warnings {
                info!("data warning: {w}");
            }
            Rc::new(data)
        }
        Err(data::LoadError::Read(e)) => anyhow::bail!("can't read the data files: {e}"),
        Err(data::LoadError::Invalid(errors)) => anyhow::bail!("the data files have errors:\n{}", errors.join("\n")),
    };
    info!(
        "data loaded and checked in {:?}: {} countries, {} suspects, {} KB of memory",
        clock.elapsed(),
        data.countries.len(),
        data.suspects.len(),
        (free - heap()) / 1024
    );

    let seed = unsafe { esp_random() } as u64;
    for language in [Lang::En, Lang::Es] {
        lang::set(language);
        info!("========== a {language:?} case, seed {seed} ==========");
        let clock = Instant::now();
        let mut game = Game::new(Rc::clone(&data), RANKS[0], Rng::new(seed), "Jon");
        info!("case set up in {:?}", clock.elapsed());
        for line in game.briefing() {
            say(&line.tone, line.speaker.as_deref(), &line.text);
        }
        play_carefully(&mut game);
        info!(
            "---------- {} at {}, deadline {}; {} flights, {} KB of memory free ----------",
            if game.won { "WON" } else { "LOST" },
            clock_text(game.t),
            clock_text(game.deadline),
            game.flights,
            heap() / 1024
        );
    }
    info!("done");
    Ok(())
}

/// Every witness in each trail city, the right flight on, the thief's full
/// profile on the warrant, then the final city's places.
fn play_carefully(game: &mut Game) {
    let mut shown = game.log.len();
    let mut show = |game: &Game, what: &str, clock: Instant| {
        info!("> {what} ({:?}, now {})", clock.elapsed(), clock_text(game.t));
        for line in &game.log[shown..] {
            say(&line.tone, line.speaker.as_deref(), &line.text);
        }
        shown = game.log.len();
    };
    for leg in 0..game.rank.legs {
        for i in 0..3 {
            let clock = Instant::now();
            game.investigate(i);
            show(game, &format!("investigate place {i} in {}", game.country(game.here).label()), clock);
        }
        let next = game.route[leg + 1];
        let clock = Instant::now();
        game.travel(next);
        show(game, &format!("fly to {}", game.country(next).label()), clock);
    }
    let thief = &game.data.suspects[game.thief];
    let fields: Vec<String> = game.warrant.iter().map(|(k, _)| k.clone()).collect();
    game.warrant = fields.iter().map(|k| (k.clone(), Some(thief.get(k).to_string()))).collect();
    let clock = Instant::now();
    game.compute_warrant();
    show(game, "compute the warrant", clock);
    for i in 0..3 {
        if game.over {
            break;
        }
        let clock = Instant::now();
        game.investigate(i);
        show(game, &format!("investigate place {i}"), clock);
    }
}

fn say(tone: &Tone, speaker: Option<&str>, text: &str) {
    match speaker {
        Some(who) => info!("  [{tone:?}] {who}: {text}"),
        None => info!("  [{tone:?}] {text}"),
    }
}

fn heap() -> u32 {
    unsafe { esp_get_free_heap_size() }
}
