//! The game rules, with no drawing code. Everything the player should read
//! goes into `Game::log`, so the same logic can run headless (tests,
//! simulations) or behind any front end.

use std::collections::HashMap;
use std::rc::Rc;

use crate::lang::{Lang, lang, tr};
use crate::rules::data::{Country, Data, SEXES, sex_shown};
use crate::rules::rng::Rng;

pub fn game_title() -> &'static str {
    tr("WHERE IN THE WORLD?", "¿DÓNDE EN EL MUNDO?")
}

pub fn agency() -> &'static str {
    tr("Family Bureau of Investigation", "Agencia Familiar de Investigaciones")
}

pub const START_HOUR: i64 = 9; // every case starts Monday 09:00
const BEDTIME: i64 = 23; // detectives sleep from 23:00 to 07:00
const WAKE_UP: i64 = 7;
pub const INTERVIEW_HOURS: i64 = 3; // each interview
pub const WARRANT_HOURS: i64 = 3; // each run of the crime computer
const FLIGHT_KMH: f64 = 800.0; // speed used to turn distance into flight time
const AIRPORT_HOURS: i64 = 1; // check-in, taxis and so on, added to each flight

#[derive(Clone, Copy)]
/// A detective's rank, as in the original: it sets how hard their cases are,
/// and they reach it after `arrests` arrests.
pub struct Rank {
    /// In English; `label` gives it in the language in use.
    pub name: &'static str,
    pub name_es: &'static str,
    pub arrests: u32,
    /// Flights from the crime scene to the hideout.
    pub legs: usize,
    /// Roughly how many wrong turns you can afford.
    pub spare_detours: usize,
    /// Extra hours on top of that.
    pub buffer: i64,
}

pub const RANKS: [Rank; 6] = [
    Rank { name: "Rookie", name_es: "Aprendiz", arrests: 0, legs: 4, spare_detours: 2, buffer: 12 },
    Rank { name: "Sleuth", name_es: "Sabueso", arrests: 1, legs: 4, spare_detours: 2, buffer: 6 },
    Rank { name: "Private Eye", name_es: "Detective privado", arrests: 3, legs: 5, spare_detours: 1, buffer: 6 },
    Rank { name: "Investigator", name_es: "Investigador", arrests: 6, legs: 5, spare_detours: 1, buffer: 2 },
    Rank { name: "Ace Detective", name_es: "Detective estrella", arrests: 10, legs: 6, spare_detours: 1, buffer: 2 },
    Rank { name: "Super Sleuth", name_es: "Supersabueso", arrests: 15, legs: 6, spare_detours: 0, buffer: 2 },
];

impl Rank {
    pub fn label(&self) -> &'static str {
        tr(self.name, self.name_es)
    }
}

/// The rank (an index into `RANKS`) of a detective with `arrests` arrests.
pub fn rank_for(arrests: u32) -> usize {
    RANKS.iter().rposition(|r| arrests >= r.arrests).unwrap_or(0)
}

/// What the printer says once an arrest has taken a detective's tally to
/// `arrests`: a promotion, or how many more arrests the next one needs.
pub fn promotion_news(arrests: u32) -> String {
    let rank = rank_for(arrests);
    let promoted = rank > rank_for(arrests.saturating_sub(1));
    let next = RANKS.get(rank + 1).map(|r| (r.label(), r.arrests - arrests));
    let now = RANKS[rank].label();
    let plural = |n: u32| match (n == 1, lang()) {
        (true, Lang::En) => "arrest",
        (false, Lang::En) => "arrests",
        (true, Lang::Es) => "arresto",
        (false, Lang::Es) => "arrestos",
    };
    match (promoted, next) {
        (true, Some((name, n))) => tr(
            format!("Congratulations! You have been promoted to {now}. {n} more {} and you'll make {name}.", plural(n)),
            format!("¡Felicitaciones! Te ascendieron a {now}. Con {n} {} más llegás a {name}.", plural(n)),
        ),
        (true, None) => tr(
            format!("Congratulations! You have been promoted to {now}, the agency's highest rank."),
            format!("¡Felicitaciones! Te ascendieron a {now}, el rango más alto de la agencia."),
        ),
        (false, Some((name, n))) => tr(
            format!("{n} more {} and you'll be promoted to {name}.", plural(n)),
            format!("Con {n} {} más te ascienden a {name}.", plural(n)),
        ),
        (false, None) => tr(
            format!("That makes {arrests} arrests for the agency's top {now}."),
            format!("Ya son {arrests} arrestos para {now}, lo más alto de la agencia."),
        ),
    }
}

pub struct Place {
    /// In English, which also picks its picture and the witness's portrait.
    pub name: &'static str,
    pub witness: &'static str,
    /// In Spanish: the name, the name with its article ("la biblioteca") and
    /// the witness, who is a man or a woman as their portrait is.
    es: (&'static str, &'static str, &'static str),
    /// Clue types this witness tends to mention.
    prefers: &'static [&'static str],
}

impl Place {
    pub fn label(&self) -> &'static str {
        tr(self.name, self.es.0)
    }

    pub fn witness_label(&self) -> &'static str {
        tr(self.witness, self.es.2)
    }

    /// "the Bank", "el banco": for "at the..." and "en...".
    fn the(&self) -> String {
        tr(format!("the {}", self.name), self.es.1.to_string())
    }
}

const fn place(
    name: &'static str,
    witness: &'static str,
    es: (&'static str, &'static str, &'static str),
    prefers: &'static [&'static str],
) -> Place {
    Place { name, witness, es, prefers }
}

pub static PLACES: [Place; 9] = [
    place("Bank", "Bank teller", ("Banco", "el banco", "Cajero"), &["currency"]),
    place("Airport", "Check-in agent", ("Aeropuerto", "el aeropuerto", "Empleada de check-in"), &["flag", "geography"]),
    place("Library", "Librarian", ("Biblioteca", "la biblioteca", "Bibliotecaria"), &["language", "landmark"]),
    place("Museum", "Museum guide", ("Museo", "el museo", "Guía del museo"), &["landmark", "culture"]),
    place("Market", "Market trader", ("Mercado", "el mercado", "Puestero"), &["culture", "currency"]),
    place("Hotel", "Hotel concierge", ("Hotel", "el hotel", "Conserje"), &[]),
    place("Bus station", "Ticket seller", ("Terminal de micros", "la terminal de micros", "Boletera"), &["geography", "landmark"]),
    place("Café", "Barista", ("Café", "el café", "Barista"), &["culture", "language"]),
    place("Taxi rank", "Taxi driver", ("Parada de taxis", "la parada de taxis", "Taxista"), &["landmark", "geography"]),
];

// What witnesses say when the thief never came through their city.
const NOBODY_ES: [&str; 6] = [
    "Perdón, no vi a nadie así por acá.",
    "No. Hoy no pasó nadie así.",
    "Me acordaría de una cara así. No la vi.",
    "Nunca vi a nadie con esa descripción, disculpá.",
    "Día tranquilo. Nada fuera de lo común.",
    "No te puedo ayudar, detective. Nadie así.",
];
const NOBODY: [&str; 6] = [
    "Sorry, I haven't seen anyone like that around here.",
    "Nope. Nobody like that came through today.",
    "I'd remember a face like that. Haven't seen one.",
    "Never seen anyone matching that description, sorry.",
    "Quiet day. Nobody unusual at all.",
    "Can't help you, detective. Nobody like that.",
];

// What witnesses say in the hideout city.
const WARNINGS: [&str; 4] = [
    "Careful, detective! Someone like that rushed past here not an hour ago. {They} must still be in town!",
    "{They} asked me about good places to hide around here. Watch your step!",
    "I saw {them} a moment ago, looking over {their} shoulder. {They} can't have gone far!",
    "{They} said {they} planned to lie low in this city for a while. Hurry!",
];

const WARNINGS_ES: [&str; 4] = [
    "¡Cuidado, detective! Alguien así pasó corriendo por acá hace menos de una hora. ¡Tiene que seguir en la ciudad!",
    "Me preguntó por buenos lugares para esconderse por acá. ¡Ojo!",
    "{Lo} vi hace un momento, mirando por encima del hombro. ¡No puede estar lejos!",
    "Dijo que pensaba quedarse escondid{o} en esta ciudad un tiempo. ¡Apurate!",
];

const SPOTTED_ES: [&str; 3] = [
    "¡ves a {name} intentando escabullirse por la puerta de atrás!",
    "¡ves a {name} escondid{o} detrás de un diario con dos agujeros para los ojos!",
    "¡ves a {name} yéndose en puntas de pie con una bolsa sospechosamente pesada!",
];

const SPOTTED: [&str; 3] = [
    "you spot {name} trying to slip out of the back door!",
    "you spot {name} hiding behind a newspaper with two eye-holes cut in it!",
    "you spot {name} tiptoeing away with a suspiciously heavy bag!",
];

const DAYS: [&str; 7] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
const DAYS_ES: [&str; 7] = ["lunes", "martes", "miércoles", "jueves", "viernes", "sábado", "domingo"];

// ------------------------------------------------------------ text helpers

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tone {
    Plain,
    Dim,
    Heading,
    Good,
    Warn,
    Bad,
}

#[derive(Clone, Debug)]
pub struct Line {
    pub tone: Tone,
    /// Set for witness statements, so a front end can style the name.
    pub speaker: Option<String>,
    pub text: String,
}

pub fn capitalise(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Swap {They}/{them}/{their}/{value} placeholders for real words, and in
/// Spanish {Él}/{él}/{lo} and the ending {o} (o or a).
pub fn fill(template: &str, pronoun: &str, value: Option<&str>) -> String {
    let (they, them, their) = match pronoun {
        "he" => ("he", "him", "his"),
        "she" => ("she", "her", "her"),
        _ => ("they", "them", "their"),
    };
    let (el, lo, o) = if pronoun == "she" { ("ella", "la", "a") } else { ("él", "lo", "o") };
    let mut out = String::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let len = after.find(|c: char| !(c.is_alphanumeric() || c == '_')).unwrap_or(after.len());
        let closed = len > 0 && after[len..].starts_with('}');
        let sub = match &after[..len] {
            _ if !closed => None,
            "they" => Some(they.to_string()),
            "them" => Some(them.to_string()),
            "their" => Some(their.to_string()),
            "They" => Some(capitalise(they)),
            "Them" => Some(capitalise(them)),
            "Their" => Some(capitalise(their)),
            "value" => value.map(str::to_string),
            "Él" => Some(capitalise(el)),
            "él" => Some(el.to_string()),
            "lo" => Some(lo.to_string()),
            "Lo" => Some(capitalise(lo)),
            "o" => Some(o.to_string()),
            _ => None,
        };
        match sub {
            Some(s) => {
                out.push_str(&s);
                rest = &after[len + 1..];
            }
            None => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    capitalise(&out)
}

/// In the original's style: "Monday, 9 a.m.", or "lunes, 9 am" (short, for
/// the small screen; the game says "9 de la mañana").
pub fn clock_text(t: i64) -> String {
    let day = (START_HOUR + t).div_euclid(24);
    let d = day.rem_euclid(7) as usize;
    let next = day >= 7;
    match lang() {
        Lang::En => {
            let time = match hour_of_day(t) {
                0 => "midnight".to_string(),
                12 => "noon".to_string(),
                h if h < 12 => format!("{h} a.m."),
                h => format!("{} p.m.", h - 12),
            };
            format!("{}{}, {time}", if next { "next " } else { "" }, DAYS[d])
        }
        Lang::Es => {
            let time = match hour_of_day(t) {
                0 => "medianoche".to_string(),
                12 => "mediodía".to_string(),
                h if h < 12 => format!("{h} am"),
                h => format!("{} pm", h - 12),
            };
            format!("{}{}, {time}", if next { "próximo " } else { "" }, DAYS_ES[d])
        }
    }
}

fn with_full_stop(mut text: String) -> String {
    if !text.ends_with('.') {
        text.push('.');
    }
    text
}

pub fn hour_of_day(t: i64) -> i64 {
    (START_HOUR + t).rem_euclid(24)
}

/// Whether the hour starting at time t is spent asleep.
pub fn asleep(t: i64) -> bool {
    !(WAKE_UP..BEDTIME).contains(&hour_of_day(t))
}

pub fn duration_text(hours: i64) -> String {
    let (days, hrs) = (hours / 24, hours % 24);
    let plural = |n: i64| if n == 1 { "" } else { "s" };
    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{days} {}{}", tr("day", "día"), plural(days)));
    }
    if hrs > 0 || days == 0 {
        parts.push(format!("{hrs} {}{}", tr("hour", "hora"), plural(hrs)));
    }
    parts.join(", ")
}

/// If it's night at time t, return the time you wake up; otherwise t.
fn after_sleep(t: i64) -> i64 {
    let hour = (START_HOUR + t).rem_euclid(24);
    if hour >= BEDTIME {
        t + (24 - hour) + WAKE_UP
    } else if hour < WAKE_UP {
        t + (WAKE_UP - hour)
    } else {
        t
    }
}

fn km_between(a: &Country, b: &Country) -> f64 {
    let (lat1, lon1, lat2, lon2) = (a.lat.to_radians(), a.lon.to_radians(), b.lat.to_radians(), b.lon.to_radians());
    let h = ((lat2 - lat1) / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * ((lon2 - lon1) / 2.0).sin().powi(2);
    2.0 * 6371.0 * h.sqrt().asin()
}

pub fn flight_hours(a: &Country, b: &Country) -> i64 {
    // round_ties_even matches Python's round(), which the timings were calibrated with.
    let flying = (km_between(a, b) / FLIGHT_KMH).round_ties_even() as i64;
    (flying + AIRPORT_HOURS).max(2)
}

pub fn same(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}

// --------------------------------------------------------------- the game

pub struct Game {
    pub data: Rc<Data>,
    pub rank: Rank,
    rng: Rng,
    /// How the briefing addresses the player, e.g. "Detective Sam".
    detective: String,
    voice: String,
    pub thief: usize,
    /// Country indices from the crime scene to the hideout.
    pub route: Vec<usize>,
    on_route: HashMap<usize, usize>,
    pub start: usize,
    /// Which of the crime scene's treasures was taken.
    treasure: usize,
    needed: Vec<String>,
    trait_plan: Vec<Vec<String>>,
    hideout: usize,
    pub deadline: i64,

    pub t: i64,
    pub here: usize,
    progress: usize, // index of the furthest route city reached
    places: HashMap<usize, [usize; 3]>,
    statements: HashMap<usize, Vec<Option<String>>>,
    visited: HashMap<usize, Vec<usize>>,
    decoys: HashMap<usize, Vec<usize>>,
    /// Crime computer entries, in display order: (key, value or unknown).
    pub warrant: Vec<(String, Option<String>)>,
    pub warrant_for: Option<usize>,
    pub over: bool,
    pub won: bool,
    /// The case ended with the thief cornered (rightly arrested or not), not with time running out.
    pub showdown: bool,
    pub flights: u32,
    pub wrong_turns: u32,
    /// Every city the player has been to, in order, for drawing the journey.
    pub path: Vec<usize>,
    /// Set by `investigate` when a V.I.L.E. henchman shows himself before an
    /// interview: which of the original's three, and whether he's the case's
    /// first (when the player is told what he means). The front end takes it.
    pub henchman: Option<(usize, bool)>,
    henchmen: u32,
    /// Set by `investigate` at a wrong place in the final city: which of the
    /// original's three warnings to show before the witness (an axe, a knife,
    /// a pistol). The front end takes it.
    pub danger: Option<usize>,
    pub log: Vec<Line>,
}

impl Game {
    pub fn new(data: Rc<Data>, rank: Rank, mut rng: Rng, detective: &str) -> Game {
        let thief = rng.below(data.suspects.len());
        let voice = data.suspects[thief].pronoun().to_string();
        let all: Vec<usize> = (0..data.countries.len()).collect();
        let route = rng.sample(&all, rank.legs + 1);
        let start = route[0];
        let treasure = rng.below(data.countries[start].treasures.len());
        let mut warrant: Vec<(String, Option<String>)> = data.traits.iter().filter(|t| t.computer).map(|t| (t.key.clone(), None)).collect();
        warrant.insert(0, ("sex".to_string(), None));
        let mut game = Game {
            on_route: route.iter().enumerate().map(|(i, &c)| (c, i)).collect(),
            data,
            rank,
            rng,
            detective: match detective.trim() {
                "" => "Detective".to_string(),
                name => format!("Detective {name}"),
            },
            voice,
            thief,
            route,
            start,
            treasure,
            needed: Vec::new(),
            trait_plan: Vec::new(),
            hideout: 0,
            deadline: 0,
            t: 0,
            here: start,
            progress: 0,
            places: HashMap::new(),
            statements: HashMap::new(),
            visited: HashMap::new(),
            decoys: HashMap::new(),
            warrant,
            warrant_for: None,
            over: false,
            won: false,
            showdown: false,
            flights: 0,
            wrong_turns: 0,
            path: vec![start],
            henchman: None,
            henchmen: 0,
            danger: None,
            log: Vec::new(),
        };
        game.needed = game.needed_traits();
        game.trait_plan = game.plan_traits();
        game.hideout = game.rng.below(3);
        game.deadline = game.compute_deadline();
        game
    }

    pub fn country(&self, i: usize) -> &Country {
        &self.data.countries[i]
    }

    /// For words that agree with the thief, like Spanish "la sospechosa".
    pub fn thief_is_female(&self) -> bool {
        self.voice == "she"
    }

    /// What was stolen, in the language in use.
    pub fn treasure(&self) -> String {
        self.country(self.start).treasures[self.treasure].get().to_string()
    }

    /// What was stolen, in English: some treasures come with a joke.
    fn treasure_en(&self) -> &str {
        &self.country(self.start).treasures[self.treasure].en
    }

    fn say(&mut self, tone: Tone, text: impl Into<String>) {
        self.log.push(Line { tone, speaker: None, text: text.into() });
    }

    fn speech(&mut self, speaker: &str, text: &str) {
        self.log.push(Line { tone: Tone::Plain, speaker: Some(speaker.to_string()), text: format!("\u{201c}{text}\u{201d}") });
    }

    // ---- case planning

    /// The crime computer's traits that single out the thief (on top of
    /// their sex, which every witness gives away), found greedily.
    fn needed_traits(&mut self) -> Vec<String> {
        let data = Rc::clone(&self.data);
        let thief = &data.suspects[self.thief];
        let mut keys = self.computer_keys();
        self.rng.shuffle(&mut keys);
        let mut pool: Vec<_> = data.suspects.iter().filter(|s| s.sex == thief.sex).collect();
        let mut needed = Vec::new();
        while pool.len() > 1 && !keys.is_empty() {
            let matching = |k: &str| pool.iter().filter(|s| same(s.get(k), thief.get(k))).count();
            let best = (0..keys.len()).min_by_key(|&i| matching(&keys[i])).unwrap_or(0);
            let best = keys.remove(best);
            pool.retain(|s| same(s.get(&best), thief.get(&best)));
            needed.push(best);
        }
        needed
    }

    fn computer_keys(&self) -> Vec<String> {
        self.data.traits.iter().filter(|t| t.computer).map(|t| t.key.clone()).collect()
    }

    /// Decide which trait each trail city's witnesses will mention.
    fn plan_traits(&mut self) -> Vec<Vec<String>> {
        let legs = self.rank.legs;
        let keys: Vec<String> = self.data.traits.iter().map(|t| t.key.clone()).collect();
        let mut slots = vec![1usize; legs];
        let target = (2 * legs).min(keys.len().max(self.needed.len()) + 1);
        while slots.iter().sum::<usize>() < target {
            let open: Vec<usize> = (0..legs).filter(|&i| slots[i] < 2).collect();
            let i = *self.rng.choice(&open);
            slots[i] += 1;
        }
        let mut seq = keys.clone();
        self.rng.shuffle(&mut seq);
        let mut extra = if self.needed.is_empty() { keys.clone() } else { self.needed.clone() };
        let total: usize = slots.iter().sum();
        while seq.len() < total {
            self.rng.shuffle(&mut extra);
            seq.extend(extra.iter().cloned());
        }
        let mut plan = Vec::new();
        let mut pos = 0;
        for n in slots {
            let mut chunk = seq[pos..pos + n].to_vec();
            pos += n;
            if chunk.len() == 2 && chunk[0] == chunk[1] {
                let others: Vec<&String> = keys.iter().filter(|k| **k != chunk[0]).collect();
                if others.is_empty() {
                    chunk.truncate(1);
                } else {
                    chunk[1] = self.rng.choice(&others).to_string();
                }
            }
            plan.push(chunk);
        }
        plan
    }

    /// Time for a careful player (every witness, a few wrong turns) plus a buffer.
    fn compute_deadline(&self) -> i64 {
        let legs: Vec<i64> =
            (0..self.rank.legs).map(|i| flight_hours(self.country(self.route[i]), self.country(self.route[i + 1]))).collect();
        let avg = (legs.iter().sum::<i64>() as f64 / legs.len() as f64).round_ties_even() as i64;
        let mut actions = Vec::new();
        for (i, &leg) in legs.iter().enumerate() {
            actions.extend([INTERVIEW_HOURS; 3]);
            if i < self.rank.spare_detours {
                actions.extend([avg, INTERVIEW_HOURS, avg]);
            }
            actions.push(leg);
        }
        actions.push(WARRANT_HOURS);
        actions.extend([INTERVIEW_HOURS; 3]);
        let mut t = 0;
        for (n, h) in actions.iter().enumerate() {
            t += h;
            if n < actions.len() - 1 {
                t = after_sleep(t);
            }
        }
        t += self.rank.buffer;
        if after_sleep(t) != t {
            // Never set a deadline in the middle of the night.
            t = after_sleep(t) + 2;
        }
        t
    }

    // ---- what witnesses say

    /// The three places (indices into PLACES) you can investigate in country `c`.
    pub fn places_in(&mut self, c: usize) -> [usize; 3] {
        if let Some(p) = self.places.get(&c) {
            return *p;
        }
        let all: Vec<usize> = (0..PLACES.len()).collect();
        let s = self.rng.sample(&all, 3);
        let picked = [s[0], s[1], s[2]];
        self.places.insert(c, picked);
        picked
    }

    fn trait_sentence(&mut self, key: &str) -> String {
        let data = Rc::clone(&self.data);
        let t = data.traits.iter().find(|t| t.key == key).expect("trait key from the data");
        let template = self.rng.choice(t.clues());
        fill(template, &self.voice, Some(data.suspects[self.thief].phrase(t)))
    }

    fn pick_clues(&mut self, country: usize, places: [usize; 3]) -> Vec<String> {
        let data = Rc::clone(&self.data);
        let mut pool: Vec<(&str, &str)> =
            data.countries[country].clues.iter().flat_map(|(cat, texts)| texts.iter().map(move |t| (cat.as_str(), t.get()))).collect();
        self.rng.shuffle(&mut pool);
        let mut picked = Vec::new();
        let mut used: Vec<&str> = Vec::new();
        for p in places {
            let prefers = PLACES[p].prefers;
            let pos = pool
                .iter()
                .position(|(cat, _)| prefers.contains(cat) && !used.contains(cat))
                .or_else(|| pool.iter().position(|(cat, _)| !used.contains(cat)))
                .unwrap_or(0);
            let (cat, text) = pool.remove(pos);
            used.push(cat);
            picked.push(text.to_string());
        }
        picked
    }

    /// What the three witnesses in country `c` say. None marks the hideout.
    fn statements(&mut self, c: usize) -> Vec<Option<String>> {
        if let Some(lines) = self.statements.get(&c) {
            return lines.clone();
        }
        let places = self.places_in(c);
        let voice = self.voice.clone();
        let lines: Vec<Option<String>> = match self.on_route.get(&c).copied() {
            None => self.rng.sample(&tr(NOBODY, NOBODY_ES), 3).into_iter().map(|s| Some(s.to_string())).collect(),
            Some(idx) if idx == self.rank.legs => {
                let mut traits = if self.needed.is_empty() { self.computer_keys() } else { self.needed.clone() };
                self.rng.shuffle(&mut traits);
                let warns = self.rng.sample(&tr(WARNINGS, WARNINGS_ES), 3);
                let mut lines = Vec::new();
                for (i, warn) in warns.iter().enumerate() {
                    if i == self.hideout {
                        lines.push(None);
                        continue;
                    }
                    let mut said = vec![fill(warn, &voice, None)];
                    if let Some(k) = traits.pop() {
                        said.push(self.trait_sentence(&k));
                    }
                    lines.push(Some(said.join(" ")));
                }
                lines
            }
            Some(idx) => {
                let clues = self.pick_clues(self.route[idx + 1], places);
                let plan = self.trait_plan[idx].clone();
                let who = self.rng.sample(&[0, 1, 2], plan.len());
                let mut lines = Vec::new();
                for (i, clue) in clues.iter().enumerate() {
                    let mut said = vec![fill(clue, &voice, None)];
                    if let Some(k) = who.iter().position(|&w| w == i) {
                        let extra = self.trait_sentence(&plan[k]);
                        let at = self.rng.below(2);
                        said.insert(at, extra);
                    }
                    lines.push(Some(said.join(" ")));
                }
                lines
            }
        };
        self.statements.insert(c, lines.clone());
        lines
    }

    /// The flights out of the current city, sorted by city name so the order gives nothing away.
    pub fn destinations(&mut self) -> Vec<usize> {
        let here = self.here;
        let idx = self.on_route.get(&here).copied();
        if !self.decoys.contains_key(&here) {
            let pool: Vec<usize> = (0..self.data.countries.len()).filter(|c| !self.on_route.contains_key(c) && *c != here).collect();
            let n = if idx == Some(self.rank.legs) { 4 } else { 3 };
            let picked = self.rng.sample(&pool, n);
            self.decoys.insert(here, picked);
        }
        let mut dests = self.decoys[&here].clone();
        match idx {
            None => dests.push(self.route[self.progress]),                  // the way back to the trail
            Some(i) if i < self.rank.legs => dests.push(self.route[i + 1]), // the right answer
            _ => {}
        }
        dests.sort_by(|a, b| self.country(*a).city().cmp(self.country(*b).city()));
        dests
    }

    /// Which of the current city's three places you've already investigated.
    pub fn visited_here(&self) -> &[usize] {
        self.visited.get(&self.here).map(Vec::as_slice).unwrap_or(&[])
    }

    // ---- actions

    pub fn briefing(&mut self) -> Vec<Line> {
        let start = self.log.len();
        let who = match self.voice.as_str() {
            "he" => tr("a man", "un hombre"),
            "she" => tr("a woman", "una mujer"),
            _ => tr("someone", "alguien"),
        };
        self.say(Tone::Heading, format!("=== {} ===", agency().to_uppercase()));
        let (place, treasure) = (self.country(self.start).label(), self.treasure());
        self.say(Tone::Warn, tr(format!("FLASH! Stolen from {place}: {treasure}!"), format!("¡URGENTE! Robo en {place}: ¡{treasure}!")));
        let saw = match self.treasure_en() {
            "Messi's trophy room" => tr(
                "After ruling out Cristiano Ronaldo as a suspect, witnesses saw",
                "Después de descartar a Cristiano Ronaldo como sospechoso, los testigos vieron a",
            ),
            _ => tr("Witnesses saw", "Los testigos vieron a"),
        };
        self.say(
            Tone::Plain,
            tr(
                format!("{saw} {who} fleeing the scene, and our files say it's someone in the family."),
                format!("{saw} {who} huyendo del lugar, y según nuestros archivos es alguien de la familia."),
            ),
        );
        // "Sunday, 9 a.m." already ends the sentence.
        let deadline = with_full_stop(clock_text(self.deadline));
        let detective = &self.detective;
        self.say(
            Tone::Plain,
            tr(
                format!(
                    "Your assignment, {detective}: follow the trail, get a warrant from the crime computer, \
                     and make the arrest before {deadline}"
                ),
                format!(
                    "Tu misión, {detective}: seguí el rastro, conseguí una orden de arresto en la computadora \
                     y hacé el arresto antes del {deadline}"
                ),
            ),
        );
        let warning = match self.treasure_en() {
            "the EVA01 from Nerv" => Some(tr("Otherwise the Third Impact will be imminent.", "Si no, el Tercer Impacto será inminente.")),
            "Sagittarius gold cloth" => Some(tr("Otherwise Saori Kido will be in trouble!", "¡Si no, Saori Kido va a estar en problemas!")),
            _ => None,
        };
        if let Some(warning) = warning {
            self.say(Tone::Plain, warning);
        }
        let lines = self.log[start..].to_vec();
        self.arrive_text();
        lines
    }

    fn arrive_text(&mut self) {
        let c = self.country(self.here);
        let (label, about) = (c.label().to_uppercase(), c.about.as_ref().map(|a| a.get().to_string()));
        self.say(Tone::Plain, "");
        self.say(Tone::Heading, label);
        self.say(Tone::Dim, clock_text(self.t));
        if let Some(about) = about {
            self.say(Tone::Plain, about);
        }
    }

    fn spend(&mut self, hours: i64) -> bool {
        self.t += hours;
        if self.t > self.deadline {
            self.time_up(false);
            return false;
        }
        true
    }

    fn night(&mut self) {
        if self.over {
            return;
        }
        let woke = after_sleep(self.t);
        if woke != self.t {
            self.t = woke;
            if self.t > self.deadline {
                self.time_up(true);
            } else {
                self.say(
                    Tone::Dim,
                    tr(
                        "It's late. You check into a hotel and sleep until 7 a.m.",
                        "Es tarde. Te registrás en un hotel y dormís hasta las 7 am.",
                    ),
                );
            }
        }
    }

    pub fn travel(&mut self, dest: usize) {
        if self.over {
            return;
        }
        let hours = flight_hours(self.country(self.here), self.country(dest));
        let (from, to) = (self.country(self.here).city(), self.country(dest).city());
        let line = tr(format!("You fly from {from} to {to} ({hours} hours)."), format!("Volás de {from} a {to} ({hours} horas)."));
        self.say(Tone::Plain, "");
        self.say(Tone::Dim, line);
        if !self.spend(hours) {
            return;
        }
        self.here = dest;
        self.path.push(dest);
        self.flights += 1;
        match self.on_route.get(&dest).copied() {
            Some(idx) if idx == self.progress + 1 => self.progress = idx,
            None => self.wrong_turns += 1,
            _ => {}
        }
        self.arrive_text();
        self.night();
    }

    /// Interview the witness at place `i` (0, 1 or 2) in the current city.
    pub fn investigate(&mut self, i: usize) {
        if self.over {
            return;
        }
        let c = self.here;
        let place = &PLACES[self.places_in(c)[i]];
        let lines = self.statements(c);
        self.say(Tone::Plain, "");
        if self.visited_here().contains(&i) {
            let at = place.the();
            self.say(
                Tone::Dim,
                tr(format!("You've already been to {at}. From your notes:"), format!("Ya estuviste en {at}. De tus notas:")),
            );
            self.speech(place.witness_label(), lines[i].as_deref().unwrap_or_default());
            return;
        }
        if !self.spend(INTERVIEW_HOURS) {
            return;
        }
        self.visited.entry(c).or_default().push(i);
        let Some(statement) = &lines[i] else {
            self.showdown(&place.the());
            return;
        };
        // As in the original: before the first interview in each city on the
        // trail the player has flown to, a henchman, so they know they're
        // right; in the final city, a weapon at every wrong place, so they know
        // the thief is close.
        let first_here = self.visited[&c].len() == 1;
        match self.on_route.get(&c) {
            Some(&idx) if idx == self.rank.legs => self.danger = Some(self.rng.below(3)),
            Some(&idx) if idx > 0 && first_here => {
                self.henchman = Some((self.rng.below(3), self.henchmen == 0));
                self.henchmen += 1;
            }
            _ => {}
        }
        let (when, at) = (clock_text(self.t), place.the());
        self.say(Tone::Dim, tr(format!("{when} at {at}"), format!("{when}, en {at}")));
        self.speech(place.witness_label(), statement);
        self.night();
    }

    pub fn fits(&self, suspect: usize) -> bool {
        let s = &self.data.suspects[suspect];
        self.warrant.iter().all(|(k, v)| v.as_ref().is_none_or(|v| same(s.get(k), v)))
    }

    pub fn compute_warrant(&mut self) {
        if self.over {
            return;
        }
        self.say(Tone::Plain, "");
        if self.warrant.iter().all(|(_, v)| v.is_none()) {
            self.say(Tone::Plain, tr("Enter at least one detail first.", "Primero ingresá al menos un dato."));
            return;
        }
        if !self.spend(WARRANT_HOURS) {
            return;
        }
        let had = self.warrant_for;
        let matches: Vec<usize> = (0..self.data.suspects.len()).filter(|&s| self.fits(s)).collect();
        if let [only] = matches[..] {
            self.warrant_for = Some(only);
            let name = &self.data.suspects[only].name;
            self.say(Tone::Good, tr(format!("Warrant issued for {name}!"), format!("¡Orden de arresto contra {name}!")));
        } else {
            self.warrant_for = None;
            if matches.is_empty() {
                self.say(
                    Tone::Warn,
                    tr(
                        "Nobody on file matches that description. Check your details.",
                        "Nadie en los archivos coincide con esa descripción. Revisá los datos.",
                    ),
                );
            } else {
                let names: Vec<&str> = matches.iter().map(|&s| self.data.suspects[s].name.as_str()).collect();
                let names = names.join(", ");
                self.say(Tone::Warn, tr(format!("Possible suspects: {names}"), format!("Posibles sospechosos: {names}")));
                self.say(Tone::Plain, tr("Add more details to narrow it down to one.", "Agregá más datos hasta que quede uno solo."));
            }
            if let Some(h) = had {
                let name = self.data.suspects[h].name.clone();
                self.say(
                    Tone::Dim,
                    tr(format!("(Your warrant for {name} has been withdrawn.)"), format!("(Se anuló tu orden de arresto contra {name}.)")),
                );
            }
        }
        self.night();
    }

    /// The crime computer's fields: (key, label, every value someone on file
    /// has). Values are as in the English data; `shown` gives them in the
    /// language in use.
    pub fn warrant_fields(&self) -> Vec<(String, String, Vec<String>)> {
        let sexes = SEXES.iter().map(|s| capitalise(s)).collect();
        let mut fields = vec![("sex".to_string(), tr("Sex", "Sexo").to_string(), sexes)];
        for t in self.data.traits.iter().filter(|t| t.computer) {
            let mut values: Vec<String> = Vec::new();
            for s in &self.data.suspects {
                let v = s.get(&t.key);
                if !values.iter().any(|x| same(x, v)) {
                    values.push(v.to_string());
                }
            }
            values.sort_by_key(|v| t.shown(v).to_lowercase());
            fields.push((t.key.clone(), t.label().to_string(), values));
        }
        fields
    }

    /// A crime computer value for field `key`, as the player reads it.
    pub fn shown(&self, key: &str, value: &str) -> String {
        if key == "sex" {
            return sex_shown(value).to_string();
        }
        match self.data.traits.iter().find(|t| t.key == key) {
            Some(t) => t.shown(value).to_string(),
            None => value.to_string(),
        }
    }

    pub fn give_up(&mut self) {
        if self.over {
            return;
        }
        self.over = true;
        let name = self.data.suspects[self.thief].name.clone();
        self.say(Tone::Plain, "");
        self.say(
            Tone::Plain,
            tr(format!("You hand in your badge for this one. The thief was {name}."), format!("Abandonás el caso. Quien robó era {name}.")),
        );
    }

    // ---- endings

    fn time_up(&mut self, asleep: bool) {
        self.over = true;
        self.t = self.t.max(self.deadline);
        let name = self.data.suspects[self.thief].name.clone();
        let treasure = self.treasure();
        self.say(Tone::Plain, "");
        let up = match (asleep, lang()) {
            (true, Lang::En) => "TIME'S UP! While you slept, the deadline passed.",
            (false, Lang::En) => "TIME'S UP! The deadline passed.",
            (true, Lang::Es) => "¡SE ACABÓ EL TIEMPO! Mientras dormías, venció el plazo.",
            (false, Lang::Es) => "¡SE ACABÓ EL TIEMPO! Venció el plazo.",
        };
        self.say(Tone::Bad, up);
        self.say(Tone::Plain, tr(format!("{name} got away with {treasure}."), format!("{name} se escapó con {treasure}.")));
    }

    fn showdown(&mut self, place: &str) {
        self.over = true;
        self.showdown = true;
        let data = Rc::clone(&self.data);
        let thief = &data.suspects[self.thief];
        let spotted = fill(self.rng.choice::<&str>(&tr(SPOTTED, SPOTTED_ES)), thief.pronoun(), None).replace("{name}", &thief.name);
        // fill() capitalises the start, which here is mid-sentence.
        let spotted = match lang() {
            Lang::En => spotted.replacen("You", "you", 1),
            Lang::Es => spotted.replacen("¡Ves", "¡ves", 1),
        };
        self.say(Tone::Bad, tr(format!("!!! At {place}, {spotted}"), format!("!!! En {place}, {spotted}")));
        let name = &thief.name;
        let here = tr(format!("{} here.", agency()), format!("Habla la {}.", agency()));
        match self.warrant_for {
            // As the original's printer put it, over the empty cell.
            None => {
                self.say(Tone::Plain, here);
                let walked = tr(
                    fill(
                        "You have trailed {name} correctly, but you had no warrant. {They} gave you a cheeky wave and walked free.",
                        thief.pronoun(),
                        None,
                    ),
                    fill(
                        "Seguiste bien el rastro de {name}, pero no tenías orden de arresto. {Él} te saludó con una sonrisa pícara y se fue tan tranquil{o}.",
                        thief.pronoun(),
                        None,
                    ),
                );
                self.say(Tone::Plain, walked.replace("{name}", name));
                self.say(Tone::Bad, tr("CASE LOST.", "CASO PERDIDO."));
            }
            Some(w) if w != self.thief => {
                self.say(Tone::Plain, here);
                let wrong = &data.suspects[w].name;
                self.say(
                    Tone::Plain,
                    tr(
                        format!("You have trailed {name} correctly, but your warrant was for {wrong}. {name} laughed and strolled off."),
                        format!("Seguiste bien el rastro de {name}, pero tu orden de arresto era contra {wrong}. {name} se rio y se fue caminando."),
                    ),
                );
                self.say(Tone::Bad, tr("CASE LOST.", "CASO PERDIDO."));
            }
            Some(_) => {
                self.won = true;
                // As the original's printer put it, after the march off to jail.
                let city = self.country(self.here).city();
                self.say(
                    Tone::Plain,
                    tr(
                        format!("Thanks to your help, the {city} police have locked up {name}."),
                        format!("Gracias a tu ayuda, la policía de {city} encerró a {name}."),
                    ),
                );
                if let Some(catchphrase) = &thief.catchphrase {
                    self.speech(&thief.name, catchphrase);
                }
                self.say(Tone::Good, tr("CASE CLOSED!", "¡CASO CERRADO!"));
                let (treasure, home, detective) = (self.treasure(), self.country(self.start).city(), &self.detective);
                self.say(
                    Tone::Plain,
                    tr(
                        format!("You recover {treasure} and send it home to {home}. Nice work, {detective}."),
                        format!("Recuperás {treasure}, que vuelve a {home}. ¡Buen trabajo, {detective}!"),
                    ),
                );
                let aftermath = match self.treasure_en() {
                    "the ball from the 7-1 defeat to Germany" => Some(tr(
                        "The Brazilian citizens, however, looked extremely sad to receive this treasure back.",
                        "Sin embargo, los brasileños se veían tristísimos de recibir de vuelta este tesoro.",
                    )),
                    "one trillion bolivares" => Some(tr(
                        "However, the one trillion bolivares are now worthless due to inflation since last week.",
                        "Sin embargo, el billón de bolívares ya no vale nada por la inflación de la semana pasada.",
                    )),
                    _ => None,
                };
                if let Some(aftermath) = aftermath {
                    self.say(Tone::Plain, aftermath);
                }
            }
        }
        let (took, flights, wrong) = (duration_text(self.t), self.flights, self.wrong_turns);
        self.say(
            Tone::Dim,
            tr(
                format!("Time taken: {took}   Flights: {flights}   Wrong turns: {wrong}"),
                format!("Tiempo: {took}   Vuelos: {flights}   Desvíos: {wrong}"),
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn data() -> Rc<Data> {
        match crate::rules::data::load(Some(Path::new(env!("CARGO_MANIFEST_DIR")))) {
            Ok((data, _)) => Rc::new(data),
            Err(_) => panic!("the data files have errors; run cargo run -- --check"),
        }
    }

    /// Follows the right route, interviews everyone, and fills in the warrant
    /// with the thief's full profile in the final city.
    fn play_perfectly(game: &mut Game) {
        for leg in 0..game.rank.legs {
            for i in 0..3 {
                game.investigate(i);
            }
            let next = game.route[leg + 1];
            assert!(game.destinations().contains(&next), "the right flight must be offered");
            game.travel(next);
        }
        let thief = &game.data.suspects[game.thief];
        let fields: Vec<_> = game.warrant.iter().map(|(k, _)| k.clone()).collect();
        game.warrant = fields.iter().map(|k| (k.clone(), Some(thief.get(k).to_string()))).collect();
        game.compute_warrant();
        for i in 0..3 {
            game.investigate(i);
        }
    }

    #[test]
    fn a_careful_player_on_the_right_trail_always_wins() {
        let data = data();
        for rank in RANKS {
            for seed in 0..300 {
                let mut game = Game::new(Rc::clone(&data), rank, Rng::new(seed), "Test");
                play_perfectly(&mut game);
                assert!(game.won, "{} seed {seed} lost: {:?}", rank.name, game.log.last());
            }
        }
    }

    #[test]
    fn the_same_seed_replays_the_same_case() {
        let data = data();
        let run = || {
            let mut game = Game::new(Rc::clone(&data), RANKS[1], Rng::new(42), "Test");
            game.briefing();
            play_perfectly(&mut game);
            game.log.iter().map(|l| l.text.clone()).collect::<Vec<_>>()
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn a_warrant_for_the_wrong_person_loses() {
        let data = data();
        let mut game = Game::new(Rc::clone(&data), RANKS[0], Rng::new(7), "Test");
        let innocent = (0..data.suspects.len()).find(|&s| s != game.thief).unwrap_or(0);
        for leg in 0..game.rank.legs {
            game.travel(game.route[leg + 1]);
        }
        game.warrant_for = Some(innocent);
        for i in 0..3 {
            game.investigate(i);
        }
        assert!(game.over && !game.won);
    }

    #[test]
    fn the_warrant_has_sex_and_only_the_computers_traits() {
        let data = data();
        let game = Game::new(Rc::clone(&data), RANKS[0], Rng::new(1), "Test");
        let keys: Vec<&str> = game.warrant.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys[0], "sex");
        for t in &data.traits {
            assert_eq!(keys.contains(&t.key.as_str()), t.computer, "{}", t.key);
        }
        let labels: Vec<String> = game.warrant_fields().into_iter().map(|(_, label, _)| label).collect();
        assert_eq!(labels[0], "Sex");
    }

    #[test]
    fn witnesses_use_a_values_phrase() {
        let data = data();
        let hair = data.traits.iter().find(|t| t.key == "hair").expect("a hair trait");
        assert_eq!(hair.phrase("Bald"), "no hair at all");
        assert_eq!(hair.phrase("ginger"), "ginger");
    }

    #[test]
    fn henchmen_show_before_the_first_interview_in_each_trail_city() {
        let mut game = Game::new(data(), RANKS[0], Rng::new(3), "Test");
        game.investigate(0);
        assert!(game.henchman.take().is_none(), "not where the case starts");
        game.travel(game.route[1]);
        game.investigate(0);
        assert!(matches!(game.henchman.take(), Some((kind, true)) if kind < 3));
        game.investigate(1);
        assert!(game.henchman.take().is_none(), "only before the first interview");
        game.travel(game.route[2]);
        game.investigate(0);
        assert!(matches!(game.henchman.take(), Some((_, false))), "the explanation comes once");
    }

    #[test]
    fn the_final_city_has_weapons_at_the_wrong_places_and_no_henchmen() {
        let mut game = Game::new(data(), RANKS[0], Rng::new(3), "Test");
        for leg in 1..=game.rank.legs {
            game.travel(game.route[leg]);
        }
        let wrong: Vec<usize> = (0..3).filter(|&i| i != game.hideout).collect();
        for i in wrong {
            game.investigate(i);
            assert!(game.henchman.take().is_none());
            assert!(matches!(game.danger.take(), Some(kind) if kind < 3));
        }
        game.investigate(game.hideout);
        assert!(game.danger.take().is_none(), "the hideout has the chase instead");
    }

    #[test]
    fn ranks_rise_with_arrests_and_say_what_comes_next() {
        assert_eq!(RANKS[rank_for(0)].name, "Rookie");
        assert_eq!(RANKS[rank_for(1)].name, "Sleuth");
        assert_eq!(RANKS[rank_for(2)].name, "Sleuth");
        assert_eq!(RANKS[rank_for(99)].name, "Super Sleuth");
        assert!(RANKS.windows(3).all(|w| w[2].arrests - w[1].arrests > w[1].arrests - w[0].arrests), "each promotion takes more arrests");
        assert_eq!(promotion_news(1), "Congratulations! You have been promoted to Sleuth. 2 more arrests and you'll make Private Eye.");
        assert_eq!(promotion_news(2), "1 more arrest and you'll be promoted to Private Eye.");
        assert!(promotion_news(15).contains("highest rank"));
        assert!(promotion_news(16).contains("16 arrests"));
    }

    #[test]
    fn fills_pronouns_and_values() {
        assert_eq!(fill("{They} showed me {their} {value}.", "she", Some("bike")), "She showed me her bike.");
        assert_eq!(fill("{they} kept {them} {unknown}", "he", None), "He kept him {unknown}");
    }

    #[test]
    fn spanish_fills_its_own_placeholders() {
        assert_eq!(fill("Dijo que pensaba quedarse escondid{o}. {Lo} vi.", "she", None), "Dijo que pensaba quedarse escondida. La vi.");
        assert_eq!(fill("{Él} tenía {value}.", "he", Some("barba")), "Él tenía barba.");
    }

    #[test]
    fn the_clock_in_spanish() {
        crate::lang::set(Lang::Es);
        assert_eq!(clock_text(0), "lunes, 9 am");
        assert_eq!(clock_text(3), "lunes, mediodía");
        assert_eq!(clock_text(8), "lunes, 5 pm");
        assert_eq!(clock_text(12), "lunes, 9 pm");
        assert_eq!(clock_text(15), "martes, medianoche");
        assert_eq!(clock_text(18), "martes, 3 am");
        assert_eq!(clock_text(24 * 7), "próximo lunes, 9 am");
        assert_eq!(duration_text(25), "1 día, 1 hora");
        crate::lang::set(Lang::En);
    }

    /// Whole cases in Spanish, won and lost: every placeholder filled and no
    /// English left in what the player reads.
    #[test]
    fn cases_read_fully_in_spanish() {
        crate::lang::set(Lang::Es);
        let data = data();
        for seed in 0..60 {
            let mut game = Game::new(Rc::clone(&data), RANKS[seed as usize % RANKS.len()], Rng::new(seed), "Test");
            game.briefing();
            if seed % 3 == 0 {
                // No warrant: the thief walks free.
                for leg in 0..game.rank.legs {
                    game.travel(game.route[leg + 1]);
                }
                for i in 0..3 {
                    game.investigate(i);
                }
            } else {
                play_perfectly(&mut game);
            }
            // Catchphrases are quoted as each person says them, in any language.
            let thief = &data.suspects[game.thief].name;
            for line in game.log.iter().filter(|l| l.speaker.as_ref() != Some(thief)) {
                assert!(!line.text.contains('{'), "seed {seed}: {}", line.text);
                for english in [" the ", "They ", " you ", "Monday", " hours"] {
                    assert!(!line.text.contains(english), "seed {seed}: {}", line.text);
                }
            }
        }
        crate::lang::set(Lang::En);
    }

    #[test]
    fn clock_and_durations() {
        assert_eq!(clock_text(0), "Monday, 9 a.m.");
        assert_eq!(clock_text(3), "Monday, noon");
        assert_eq!(clock_text(8), "Monday, 5 p.m.");
        assert_eq!(clock_text(15), "Tuesday, midnight");
        assert_eq!(clock_text(24 * 7), "next Monday, 9 a.m.");
        assert_eq!(duration_text(25), "1 day, 1 hour");
        assert_eq!(duration_text(0), "0 hours");
        assert_eq!(after_sleep(14), 22); // 23:00 Monday -> 07:00 Tuesday
    }
}
