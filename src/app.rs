//! The game on the screen: headquarters (the name, typed on an on-screen
//! keyboard), the briefing, the case (the four buttons: notes, flights,
//! places to investigate, the crime computer), the ending, and the next case.
//!
//! Input is a press and a release: `press` highlights what's under the
//! finger, `release` acts on it, and `draw` paints the whole screen from the
//! state. Between touches only scenes change the screen: `tick` moves them
//! on, and says what to redraw. Sounds are left in `take_cues` for the main
//! loop to play.

use std::rc::Rc;
use std::time::Instant;

use crate::art::pixels::{Canvas, Rgb};
use crate::art::postcard;
use crate::art::scene::TimeOfDay;
use crate::art::sprites;
use crate::device::net::Net;
use crate::device::progress::{self, Detective, MAX_NAME, News, Progress};
use crate::device::sound::{self, Notes};
use crate::display::{H, W};
use crate::lang::{self, Lang, lang, tr};
use crate::rules::data::{Data, sex_shown};
use crate::rules::game::{Game, Line, RANKS, Tone, agency, clock_text, game_title, hour_of_day, promotion_news, rank_for};
use crate::rules::game::{PLACES, same};
use crate::rules::rng::Rng;
use crate::scenes::Stage;
use crate::ui::{self, BUTTON_ORDER, BUTTONS, Button, CLOCK, Fonts, PAD, PICTURE, Pager, Rect, Row, TEXT, rect};

/// Something the finger can press.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Target {
    Button(Button),
    /// A row of a list in the text panel: a place, a flight, a crime computer field.
    Row(usize),
    /// A key of the on-screen keyboard.
    Key(usize),
    /// The text panel itself, to turn the page.
    Panel,
    /// The clock box: switches language.
    Clock,
    /// The detective roster: a button at headquarters, half the crime computer's last row.
    Roster,
    /// Everyone's files: the other half of that row.
    Files,
}

enum Panel {
    /// Headquarters: the keyboard for the detective's name.
    Name,
    /// Text to read, a page at a time; then `then`.
    Story { pager: Pager, then: Then },
    /// The three places to investigate here (indices into `PLACES`).
    Places([usize; 3]),
    /// Where the flights from here go.
    Flights(Vec<usize>),
    /// The crime computer's fields, and its compute row.
    Crime,
    /// The case notes: deadline, warrant and everything said so far.
    Notes(Pager),
    /// Animated moments, one after another, then what happened.
    Scene(Scene),
    /// Asking the bureau for the detective's arrests as they sign in.
    Checking(Instant),
    /// Every detective on file; `online` is None while the server's asked.
    Roster { pager: Pager, from_hq: bool, online: Option<bool>, asked: Instant },
    /// One suspect's file, `who` in the data's order.
    Dossier { who: usize, pager: Pager },
}

struct Scene {
    /// The one playing first.
    stages: Vec<Stage>,
    started: Instant,
    /// How many of its cues have been played.
    cued: usize,
    then: Vec<Line>,
}

/// What a `tick` changed.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Redraw {
    Nothing,
    /// Only the text panel.
    Panel,
    Everything,
}

#[derive(Clone, Copy, PartialEq)]
enum Then {
    /// Stay: the buttons say what next.
    Stay,
    /// The briefing's over: on to the first city.
    Arrive,
    /// The case is over: back to headquarters for another.
    NextCase,
}

/// The keyboard: letters, Ñ, delete, the language and OK, six to a row.
const KEYS: [&str; 30] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "Ñ", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
    "⌫", "LANG", "OK",
];
const KEY_COLUMNS: i32 = 6;

/// Rows of a list in the text panel: a heading, then rows this tall.
const LIST_ROW: i32 = 22;
/// How long signing in waits for the bureau, as the desktop does.
const CHECK_SECONDS: f64 = 4.0;
/// How long the roster waits for the bureau before showing this device's records.
const ROSTER_SECONDS: f64 = 12.0;

pub struct App {
    data: Rc<Data>,
    fonts: Fonts,
    progress: Progress,
    seed: fn() -> u64,
    name: String,
    game: Option<Game>,
    panel: Panel,
    pressed: Option<Target>,
    /// The postcard on show, painted once per city and time of day.
    postcard: Option<((usize, TimeOfDay), Canvas)>,
    /// Sounds to play (None to stop), for the main loop.
    cues: Vec<Option<Notes>>,
}

impl App {
    /// Starts at headquarters. `seed` gives each new case its randomness;
    /// `net` reaches the bureau's server, if there's WiFi.
    pub fn new(data: Rc<Data>, seed: fn() -> u64, net: Option<Net>) -> App {
        App {
            data,
            fonts: Fonts::new(),
            progress: Progress::load(net),
            seed,
            name: String::new(),
            game: None,
            panel: Panel::Name,
            pressed: None,
            postcard: None,
            cues: Vec::new(),
        }
    }

    /// Sounds the game wants played since last asked, in order; None stops the sound.
    pub fn take_cues(&mut self) -> Vec<Option<Notes>> {
        std::mem::take(&mut self.cues)
    }

    /// Whether something moves on by itself: a scene, or a wait for the bureau.
    pub fn animating(&self) -> bool {
        matches!(self.panel, Panel::Scene(_) | Panel::Checking(_) | Panel::Roster { online: None, .. })
    }

    /// Takes in the bureau's answers. Call often.
    pub fn poll(&mut self) -> Redraw {
        let mut redraw = Redraw::Nothing;
        for news in self.progress.poll() {
            match news {
                News::LookedUp if matches!(self.panel, Panel::Checking(_)) => {
                    self.new_case();
                    redraw = Redraw::Everything;
                }
                News::Roster(list) if matches!(self.panel, Panel::Roster { online: None, .. }) => {
                    self.show_roster(list);
                    redraw = Redraw::Everything;
                }
                _ => {}
            }
        }
        redraw
    }

    /// Moves a scene on: plays its cues as they come due, and goes to the
    /// next stage, or to what happened, when one ends. Gives up on the
    /// bureau when it's slow.
    pub fn tick(&mut self) -> Redraw {
        match &self.panel {
            Panel::Checking(since) if since.elapsed().as_secs_f64() >= CHECK_SECONDS => {
                self.new_case();
                return Redraw::Everything;
            }
            Panel::Roster { online: None, asked, .. } if asked.elapsed().as_secs_f64() >= ROSTER_SECONDS => {
                self.show_roster(None);
                return Redraw::Everything;
            }
            _ => {}
        }
        let Panel::Scene(scene) = &mut self.panel else { return Redraw::Nothing };
        let t = scene.started.elapsed().as_secs_f64();
        let cues = scene.stages[0].cues();
        while let Some(&(at, notes)) = cues.get(scene.cued)
            && at <= t
        {
            self.cues.push(Some(notes));
            scene.cued += 1;
        }
        if t < scene.stages[0].seconds(scene_area()) {
            return Redraw::Panel;
        }
        self.next_stage();
        if self.animating() { Redraw::Panel } else { Redraw::Everything }
    }

    /// Ends the scene's current stage: on to the next, or to what happened.
    fn next_stage(&mut self) {
        let Panel::Scene(scene) = &mut self.panel else { return };
        self.cues.push(None);
        scene.stages.remove(0);
        if scene.stages.is_empty() {
            let then = std::mem::take(&mut scene.then);
            self.tell(then);
        } else {
            scene.started = Instant::now();
            scene.cued = 0;
        }
    }

    /// Plays `stages`, then shows `then`.
    fn play(&mut self, stages: Vec<Stage>, then: Vec<Line>) {
        if stages.is_empty() {
            self.tell(then);
        } else {
            self.panel = Panel::Scene(Scene { stages, started: Instant::now(), cued: 0, then });
        }
    }

    // ------------------------------------------------------------ input

    /// The finger went down at (x, y); true if the screen should change.
    pub fn press(&mut self, x: i32, y: i32) -> bool {
        self.pressed = self.target_at(x, y);
        self.pressed.is_some()
    }

    /// The finger came up: act on what it pressed. True if the screen should change.
    pub fn release(&mut self) -> bool {
        let Some(target) = self.pressed.take() else { return false };
        self.act(target);
        true
    }

    fn buttons_on(&self) -> bool {
        self.game.as_ref().is_some_and(|g| !g.over)
            && !matches!(self.panel, Panel::Name | Panel::Story { then: Then::Arrive, .. } | Panel::Scene(_) | Panel::Checking(_))
    }

    fn target_at(&self, x: i32, y: i32) -> Option<Target> {
        if CLOCK.contains(x, y) {
            return Some(Target::Clock);
        }
        if let Some(i) = BUTTONS.iter().position(|b| b.contains(x, y)) {
            return self.buttons_on().then_some(Target::Button(BUTTON_ORDER[i]));
        }
        if matches!(self.panel, Panel::Name) && hq_roster_button().contains(x, y) {
            return Some(Target::Roster);
        }
        if !TEXT.contains(x, y) {
            return None;
        }
        let area = text_area();
        match &self.panel {
            Panel::Name => {
                let (kx, ky) = ((x - area.x) / key_size().0, (y - keyboard_top()) / key_size().1);
                let k = (ky * KEY_COLUMNS + kx) as usize;
                (y >= keyboard_top() && kx < KEY_COLUMNS && k < KEYS.len()).then_some(Target::Key(k))
            }
            Panel::Story { .. } | Panel::Notes(_) | Panel::Scene(_) | Panel::Roster { .. } | Panel::Dossier { .. } => Some(Target::Panel),
            Panel::Checking(_) => None,
            Panel::Places(_) => self.list_row(y, 3),
            Panel::Flights(dests) => self.list_row(y, dests.len()),
            Panel::Crime => {
                // The fields, COMPUTE, then a row split between the files and the roster.
                let fields = self.game.as_ref().map_or(0, |g| g.warrant.len());
                match self.list_row(y, fields + 2) {
                    Some(Target::Row(i)) if i == fields + 1 => Some(if x < area.x + area.w / 2 { Target::Files } else { Target::Roster }),
                    other => other,
                }
            }
        }
    }

    /// Which of `n` list rows is at height `y`.
    fn list_row(&self, y: i32, n: usize) -> Option<Target> {
        let top = list_top(&self.fonts);
        let row = (y - top).div_euclid(LIST_ROW);
        (y >= top && (row as usize) < n).then_some(Target::Row(row as usize))
    }

    fn act(&mut self, target: Target) {
        match target {
            Target::Clock => {
                lang::choose(if lang() == Lang::En { Lang::Es } else { Lang::En });
            }
            Target::Key(k) => {
                self.cues.push(Some(sound::KEY));
                self.key(k);
            }
            // A tap skips the rest of a scene's stage.
            Target::Panel if self.animating() => self.next_stage(),
            Target::Panel => {
                self.cues.push(Some(sound::TICK));
                self.turn_page();
            }
            Target::Button(b) => {
                self.cues.push(Some(sound::TICK));
                self.button(b);
            }
            Target::Row(i) => self.row(i),
            Target::Roster => {
                self.cues.push(Some(sound::TICK));
                self.ask_roster();
            }
            Target::Files => {
                self.cues.push(Some(sound::TICK));
                self.dossier(0);
            }
        }
    }

    fn key(&mut self, k: usize) {
        match KEYS[k] {
            "⌫" => {
                self.name.pop();
            }
            "LANG" => lang::choose(if lang() == Lang::En { Lang::Es } else { Lang::En }),
            "OK" => {
                if !self.name.trim().is_empty() {
                    // The bureau may know them from another device; don't wait long.
                    if self.progress.look_up(&self.name) {
                        self.panel = Panel::Checking(Instant::now());
                    } else {
                        self.new_case();
                    }
                }
            }
            letter => {
                if self.name.chars().count() < MAX_NAME {
                    // Typed in capitals, kept as a name: Jon, not JON.
                    let c = if self.name.is_empty() { letter.to_string() } else { letter.to_lowercase() };
                    self.name.push_str(&c);
                }
            }
        }
    }

    fn turn_page(&mut self) {
        let (f, area) = (&self.fonts.body, text_area());
        match &mut self.panel {
            Panel::Story { pager, then } => {
                if pager.turn(f, area) {
                    return;
                }
                match *then {
                    Then::Stay => {}
                    Then::Arrive => self.arrive(),
                    Then::NextCase => self.new_case(),
                }
            }
            Panel::Notes(pager) => {
                if !pager.turn(f, area) {
                    pager.page = 0;
                }
            }
            Panel::Roster { pager, from_hq, .. } => {
                if !pager.turn(f, area) {
                    self.panel = if *from_hq { Panel::Name } else { Panel::Crime };
                }
            }
            Panel::Dossier { who, pager } => {
                // After the last page of a file, the next suspect's.
                if !pager.turn(f, area) {
                    let next = (*who + 1) % self.data.suspects.len();
                    self.dossier(next);
                }
            }
            _ => {}
        }
    }

    /// Asks the bureau for the roster; this device's records if there's no server.
    fn ask_roster(&mut self) {
        let from_hq = self.game.is_none();
        if self.progress.ask_roster() {
            let waiting = ui::rows(&self.fonts.body, tr("Asking the bureau…", "Consultando a la agencia…"), ui::GREY, text_area().w);
            self.panel = Panel::Roster { pager: Pager::new(waiting), from_hq, online: None, asked: Instant::now() };
        } else {
            self.panel = Panel::Roster { pager: Pager::default(), from_hq, online: Some(false), asked: Instant::now() };
            self.show_roster(None);
        }
    }

    /// The roster: `list` from the bureau, or this device's records if None.
    fn show_roster(&mut self, list: Option<Vec<Detective>>) {
        let Panel::Roster { from_hq, .. } = self.panel else { return };
        let online = list.is_some();
        let list = list.unwrap_or_else(|| self.progress.local_roster());
        let (f, width) = (&self.fonts.body, text_area().w);
        let mut rows = ui::rows(f, tr("DETECTIVE ROSTER", "LISTA DE DETECTIVES"), ui::YELLOW, width);
        if !online {
            rows.extend(ui::rows(
                f,
                tr("(This device only: no answer from the bureau.)", "(Solo este equipo: la agencia no responde.)"),
                ui::GREY,
                width,
            ));
        }
        rows.push(Row::gap());
        let me = progress::key(&self.name);
        for d in &list {
            let mine = me.is_some() && progress::key(&d.name) == me;
            let mut name: Vec<char> = d.name.chars().collect();
            if let Some(first) = name.first_mut() {
                *first = first.to_uppercase().next().unwrap_or(*first);
            }
            let name: String = name.into_iter().collect();
            rows.extend(ui::rows(f, &name, if mine { ui::YELLOW } else { ui::WHITE }, width));
            let rank = RANKS[rank_for(d.arrests)].label();
            let arrests = tr(format!("  {rank}, {} arrests", d.arrests), format!("  {rank}, {} arrestos", d.arrests));
            rows.extend(ui::rows(f, &arrests, ui::GREY, width));
        }
        if list.is_empty() {
            rows.extend(ui::rows(f, tr("Nobody on file yet.", "Todavía no hay nadie."), ui::GREY, width));
        }
        self.panel = Panel::Roster { pager: Pager::new(rows), from_hq, online: Some(online), asked: Instant::now() };
    }

    /// Suspect `who`'s file: the photo in the picture frame, the facts in the panel.
    fn dossier(&mut self, who: usize) {
        let s = &self.data.suspects[who];
        let (f, width) = (&self.fonts.body, text_area().w);
        let mut rows = ui::rows(f, &s.name.to_uppercase(), ui::YELLOW, width);
        let mut facts = vec![(tr("Sex", "Sexo").to_string(), sex_shown(&s.sex).to_string())];
        for t in &self.data.traits {
            facts.push((t.label().to_string(), s.shown(t).to_string()));
        }
        if let Some(bio) = &s.bio {
            facts.push((tr("Notes", "Notas").to_string(), bio.get().to_string()));
        }
        for (label, value) in facts {
            let mut lines = ui::rows(f, &format!("{label}: {value}"), ui::WHITE, width);
            if let Some(first) = lines.first_mut() {
                first.speaker = (label.chars().count() + 1).min(first.text.chars().count());
            }
            rows.extend(lines);
        }
        if let Some(said) = &s.catchphrase {
            rows.extend(ui::rows(f, &format!("\u{201c}{said}\u{201d}"), ui::LIME, width));
        }
        let n = self.data.suspects.len();
        rows.extend(ui::rows(
            f,
            &tr(format!("File {} of {n}: tap for the next.", who + 1), format!("Ficha {} de {n}: tocá para la siguiente.", who + 1)),
            ui::GREY,
            width,
        ));
        self.panel = Panel::Dossier { who, pager: Pager::new(rows) };
    }

    fn button(&mut self, b: Button) {
        let Some(game) = self.game.as_mut() else { return };
        self.panel = match b {
            Button::Notes => {
                let mut pager = Pager::new(notes_rows(&self.fonts, game));
                // Open on the newest page; a tap goes on round from the oldest.
                pager.page = pager.pages(&self.fonts.body, text_area()) - 1;
                Panel::Notes(pager)
            }
            Button::Depart => Panel::Flights(game.destinations()),
            Button::Investigate => Panel::Places(game.places_in(game.here)),
            Button::Crime => Panel::Crime,
        };
    }

    fn row(&mut self, i: usize) {
        let Some(game) = self.game.as_mut() else { return };
        let before = game.log.len();
        let mut stages = Vec::new();
        match &self.panel {
            Panel::Places(places) => {
                // Places already visited are no walk away.
                if !game.visited_here().contains(&i) {
                    stages.push(Stage::Walk(PLACES[places[i]].label().to_string()));
                }
                game.investigate(i);
                if let Some((kind, first)) = game.henchman.take() {
                    stages.push(Stage::Henchman { kind, tell: first });
                }
                if let Some(kind) = game.danger.take() {
                    stages.push(Stage::Danger(kind));
                }
                if game.over && game.showdown {
                    stages.push(Stage::Chase { female: game.thief_is_female() });
                    stages.push(Stage::March { caught: game.won });
                    stages.push(Stage::Jail { caught: game.won });
                }
            }
            Panel::Flights(dests) => {
                stages.push(Stage::Fly(game.country(dests[i]).city().to_string()));
                game.travel(dests[i]);
            }
            Panel::Crime => {
                if i == game.warrant.len() {
                    stages.push(Stage::Compute);
                    game.compute_warrant();
                } else {
                    let fields = game.warrant_fields();
                    let (_, _, values) = &fields[i];
                    let now = game.warrant[i].1.clone();
                    // Each tap steps to the next value, and from the last back to unknown.
                    let next = match now.and_then(|v| values.iter().position(|x| same(x, &v))) {
                        None => values.first().cloned(),
                        Some(p) => values.get(p + 1).cloned(),
                    };
                    game.warrant[i].1 = next;
                    self.cues.push(Some(sound::KEY));
                    return;
                }
            }
            _ => return,
        }
        let then = game.log[before..].to_vec();
        self.play(stages, then);
    }

    // ------------------------------------------------------------ the story

    /// Shows what just happened; if the case ended with it, counts an arrest
    /// and says what it means for the detective's rank.
    fn tell(&mut self, mut lines: Vec<Line>) {
        // Everything said goes to the serial monitor too, to look back on.
        for line in &lines {
            match &line.speaker {
                Some(who) => log::info!("[{:?}] {who}: {}", line.tone, line.text),
                None => log::info!("[{:?}] {}", line.tone, line.text),
            }
        }
        let game = self.game.as_ref().expect("a case");
        let mut then = Then::Stay;
        if game.over {
            then = Then::NextCase;
            if game.won {
                let arrests = self.progress.add_arrest(&self.name);
                lines.push(said(Tone::Plain, ""));
                lines.push(said(Tone::Good, &promotion_news(arrests)));
            }
        }
        self.panel = Panel::Story { pager: Pager::new(ui::log_rows(&self.fonts.body, &lines, text_area().w)), then };
    }

    /// A new case for the detective at their rank, with the welcome and briefing.
    fn new_case(&mut self) {
        let arrests = self.progress.arrests(&self.name);
        let rank = RANKS[rank_for(arrests)];
        let mut game = Game::new(Rc::clone(&self.data), rank, Rng::new((self.seed)()), self.name.trim());
        let welcome = if arrests == 0 && self.game.is_none() {
            tr(format!("Welcome, {}.", self.name), format!("Te damos la bienvenida, {}.", self.name))
        } else {
            tr(format!("Welcome back, {}.", self.name), format!("Hola de nuevo, {}.", self.name))
        };
        let mut lines = vec![
            said(Tone::Plain, &welcome),
            said(Tone::Plain, &tr(format!("Your rank: {}.", rank.label()), format!("Tu rango: {}.", rank.label()))),
            said(Tone::Plain, ""),
        ];
        lines.extend(game.briefing());
        self.game = Some(game);
        self.panel = Panel::Story { pager: Pager::new(ui::log_rows(&self.fonts.body, &lines, text_area().w)), then: Then::Arrive };
    }

    /// The first city: its name and what it's like.
    fn arrive(&mut self) {
        let game = self.game.as_ref().expect("a case");
        let country = game.country(game.here);
        let mut lines = vec![said(Tone::Heading, &country.label().to_uppercase())];
        if let Some(about) = &country.about {
            lines.push(said(Tone::Plain, about.get()));
        }
        self.panel = Panel::Story { pager: Pager::new(ui::log_rows(&self.fonts.body, &lines, text_area().w)), then: Then::Stay };
    }

    // ------------------------------------------------------------ drawing

    pub fn draw(&mut self) -> Canvas {
        let mut cv = Canvas::new(W as i32, H as i32);
        self.draw_clock(&mut cv);
        self.draw_picture(&mut cv);
        self.draw_panel(&mut cv);
        let on = self.buttons_on();
        for (i, &b) in BUTTON_ORDER.iter().enumerate() {
            ui::button(&mut cv, &self.fonts, b, BUTTONS[i], self.pressed == Some(Target::Button(b)), on);
        }
        cv
    }

    fn draw_clock(&self, cv: &mut Canvas) {
        ui::frame(cv, CLOCK, if self.pressed == Some(Target::Clock) { ui::YELLOW } else { ui::WHITE });
        let c = CLOCK.inner();
        let f = &self.fonts.body;
        let text = match &self.game {
            Some(g) => clock_text(g.t),
            None => tr("Headquarters", "Cuartel general").to_string(),
        };
        let text = f.wrap(&text, c.w - 2 * PAD).into_iter().next().unwrap_or_default();
        f.draw(cv, c.x + PAD, c.y + (c.h - f.line_height()) / 2 + f.ascent(), &text, ui::WHITE);
        let tag = if lang() == Lang::En { "EN" } else { "ES" };
        self.fonts.label.draw(cv, c.x + c.w - self.fonts.label.width(tag) - 1, c.y + self.fonts.label.ascent(), tag, ui::GREY);
    }

    fn draw_picture(&mut self, cv: &mut Canvas) {
        ui::frame(cv, PICTURE, ui::WHITE);
        let p = PICTURE.inner();
        let Some(game) = &self.game else {
            // Headquarters: the title at the top; the roster's button at the
            // bottom, the name as it's typed above it and the prompt above
            // that; and the agency in between, if there's room.
            let f = &self.fonts;
            let width = p.w - 2 * PAD;
            let b = hq_roster_button();
            ui::frame(cv, b, if self.pressed == Some(Target::Roster) { ui::YELLOW } else { ui::WHITE });
            let label = tr("DETECTIVE ROSTER", "LISTA DE DETECTIVES");
            f.label.draw(
                cv,
                b.x + (b.w - f.label.width(label)) / 2,
                b.y + (b.h - f.label.line_height()) / 2 + f.label.ascent(),
                label,
                ui::WHITE,
            );
            let name_top = b.y - 6 - f.heading.line_height();
            f.heading.draw(cv, p.x + PAD, name_top + f.heading.ascent(), &format!("{}_", self.name), ui::CYAN);
            let prompt = f.body.wrap(tr("Detective at keyboard, please identify yourself:", "Detective al teclado, identificate:"), width);
            let prompt_top = name_top - 2 - prompt.len() as i32 * f.body.line_height();
            for (k, line) in prompt.iter().enumerate() {
                f.body.draw(cv, p.x + PAD, prompt_top + k as i32 * f.body.line_height() + f.body.ascent(), line, ui::WHITE);
            }
            let mut y = p.y + PAD;
            for line in f.heading.wrap(game_title(), width) {
                f.heading.draw(cv, p.x + PAD, y + f.heading.ascent(), &line, ui::YELLOW);
                y += f.heading.line_height();
            }
            let agency = f.body.wrap(&agency().to_uppercase(), width);
            if y + 2 + agency.len() as i32 * f.body.line_height() <= prompt_top - 4 {
                for (k, line) in agency.iter().enumerate() {
                    f.body.draw(cv, p.x + PAD, y + 2 + k as i32 * f.body.line_height() + f.body.ascent(), line, ui::GREY);
                }
            }
            return;
        };
        if let Panel::Dossier { who, .. } = &self.panel {
            // The photo at double size nearly fills the frame, as on the original's sheet.
            cv.rect(p.x, p.y, p.w, p.h, ui::BLACK);
            if sprites::photo(cv, &self.data.suspects[*who].name, p.x + (p.w - 132) / 2, p.y + (p.h - 192) / 2, 2) {
                return;
            }
        }
        let key = (game.here, TimeOfDay::at(hour_of_day(game.t)));
        if self.postcard.as_ref().is_none_or(|(k, _)| *k != key) {
            let country = game.country(game.here);
            self.postcard = Some((key, postcard::fitted(&country.country, country.lat, key.1, p.w, p.h)));
        }
        if let Some((_, card)) = &self.postcard {
            for (i, &px) in card.pixels().iter().enumerate() {
                cv.set(p.x + i as i32 % p.w, p.y + i as i32 / p.w, px);
            }
        }
    }

    fn draw_panel(&self, cv: &mut Canvas) {
        ui::frame(cv, TEXT, ui::WHITE);
        let inner = TEXT.inner();
        cv.rect(inner.x, inner.y, inner.w, inner.h, ui::NAVY);
        let area = text_area();
        let f = &self.fonts;
        match &self.panel {
            Panel::Name => self.draw_keyboard(cv),
            Panel::Story { pager, .. } | Panel::Notes(pager) => pager.draw(cv, &f.body, area),
            Panel::Scene(scene) => scene.stages[0].draw(cv, f, scene_area(), scene.started.elapsed().as_secs_f64()),
            Panel::Roster { pager, .. } | Panel::Dossier { pager, .. } => pager.draw(cv, &f.body, area),
            Panel::Checking(_) => {
                for (k, line) in
                    f.body.wrap(tr("Checking the bureau's files…", "Revisando los archivos de la agencia…"), area.w).iter().enumerate()
                {
                    f.body.draw(cv, area.x, area.y + k as i32 * f.body.line_height() + f.body.ascent(), line, ui::GREY);
                }
            }
            Panel::Places(places) => {
                let game = self.game.as_ref().expect("a case");
                let visited = game.visited_here();
                let rows: Vec<(String, Rgb)> = places
                    .iter()
                    .enumerate()
                    .map(|(i, &p)| (PLACES[p].label().to_string(), if visited.contains(&i) { ui::GREY } else { ui::WHITE }))
                    .collect();
                self.draw_list(cv, tr("Where to investigate?", "¿Dónde investigar?"), &rows);
            }
            Panel::Flights(dests) => {
                let game = self.game.as_ref().expect("a case");
                let rows: Vec<(String, Rgb)> = dests.iter().map(|&d| (game.country(d).label(), ui::WHITE)).collect();
                self.draw_list(cv, tr("Fly to:", "Volar a:"), &rows);
            }
            Panel::Crime => {
                let game = self.game.as_ref().expect("a case");
                let mut rows: Vec<(String, Rgb)> = game
                    .warrant_fields()
                    .iter()
                    .zip(&game.warrant)
                    .map(|((key, label, _), (_, value))| match value {
                        Some(v) => (format!("{label}: {}", game.shown(key, v)), ui::WHITE),
                        None => (format!("{label}: ?"), ui::GREY),
                    })
                    .collect();
                rows.push((tr("COMPUTE", "CALCULAR").to_string(), ui::YELLOW));
                self.draw_list(cv, tr("Crime computer", "Computadora"), &rows);
                // The last row, split: the files on the left, the roster on the right.
                let y = list_top(f) + rows.len() as i32 * LIST_ROW;
                let half = area.w / 2;
                for (k, (label, target)) in
                    [(tr("FILES", "FICHAS"), Target::Files), (tr("ROSTER", "LISTA"), Target::Roster)].into_iter().enumerate()
                {
                    let x = area.x + k as i32 * half;
                    if self.pressed == Some(target) {
                        cv.rect(x, y, half - 2, LIST_ROW - 2, ui::BLUE);
                    }
                    cv.rect(x, y + LIST_ROW - 2, half - 2, 1, ui::DARK);
                    f.body.draw(cv, x + 1, y + (LIST_ROW - f.body.line_height()) / 2 + f.body.ascent(), label, ui::CYAN);
                }
            }
        }
    }

    /// A heading, then rows to tap, the pressed one highlighted.
    fn draw_list(&self, cv: &mut Canvas, heading: &str, rows: &[(String, Rgb)]) {
        let (f, area) = (&self.fonts.body, text_area());
        f.draw(cv, area.x, area.y + f.ascent(), heading, ui::YELLOW);
        let top = list_top(&self.fonts);
        for (i, (text, colour)) in rows.iter().enumerate() {
            let y = top + i as i32 * LIST_ROW;
            if self.pressed == Some(Target::Row(i)) {
                cv.rect(area.x - 1, y, area.w + 2, LIST_ROW - 2, ui::BLUE);
            }
            cv.rect(area.x, y + LIST_ROW - 2, area.w, 1, ui::DARK);
            // A long name is cut short to keep each row one line.
            let text = f.wrap(text, area.w - 2).into_iter().next().unwrap_or_default();
            f.draw(cv, area.x + 1, y + (LIST_ROW - f.line_height()) / 2 + f.ascent(), &text, *colour);
        }
    }

    fn draw_keyboard(&self, cv: &mut Canvas) {
        let (f, area) = (&self.fonts, text_area());
        let (kw, kh) = key_size();
        let top = keyboard_top();
        f.body.draw(cv, area.x, area.y + f.body.ascent(), tr("Your name:", "Tu nombre:"), ui::YELLOW);
        for (k, &label) in KEYS.iter().enumerate() {
            let (col, row) = (k as i32 % KEY_COLUMNS, k as i32 / KEY_COLUMNS);
            let key = rect(area.x + col * kw, top + row * kh, kw - 2, kh - 2);
            let pressed = self.pressed == Some(Target::Key(k));
            cv.rect(key.x, key.y, key.w, key.h, if pressed { ui::YELLOW } else { ui::BLUE });
            let label = match label {
                "LANG" => tr("ES", "EN"),
                "⌫" => "<",
                l => l,
            };
            let font = if label.chars().count() > 1 { &f.label } else { &f.body };
            let ink = if pressed { ui::NAVY } else { ui::WHITE };
            font.draw(cv, key.x + (key.w - font.width(label)) / 2, key.y + (key.h - font.line_height()) / 2 + font.ascent(), label, ink);
        }
    }
}

/// Where scenes play: the whole of the text panel inside its frame, so the
/// characters stand on its bottom edge.
fn scene_area() -> Rect {
    TEXT.inner()
}

/// The roster's button at headquarters, at the bottom of the picture frame.
fn hq_roster_button() -> Rect {
    let p = PICTURE.inner();
    rect(p.x + PAD, p.y + p.h - 30, p.w - 2 * PAD, 26)
}

/// Where the text panel's text goes.
fn text_area() -> Rect {
    let i = TEXT.inner();
    rect(i.x + PAD, i.y + PAD, i.w - 2 * PAD, i.h - 2 * PAD)
}

fn list_top(f: &Fonts) -> i32 {
    text_area().y + f.body.line_height() + 4
}

fn key_size() -> (i32, i32) {
    let area = text_area();
    let rows = KEYS.len().div_ceil(KEY_COLUMNS as usize) as i32;
    (area.w / KEY_COLUMNS, (area.y + area.h - keyboard_top()) / rows)
}

fn keyboard_top() -> i32 {
    text_area().y + 20
}

/// The notes: the deadline, the warrant, then everything said in the case.
fn notes_rows(f: &Fonts, game: &Game) -> Vec<Row> {
    let width = text_area().w;
    let left = (game.deadline - game.t).max(0);
    let due = clock_text(game.deadline);
    let mut lines = vec![
        said(Tone::Heading, tr("CASE NOTES", "NOTAS DEL CASO")),
        said(
            if left <= 12 { Tone::Bad } else { Tone::Plain },
            &tr(format!("Deadline {due} ({left} hours left)"), format!("Vence el {due} ({left} h)")),
        ),
        said(
            Tone::Dim,
            &match game.warrant_for {
                Some(s) => tr(format!("Warrant: {}", game.data.suspects[s].name), format!("Orden: {}", game.data.suspects[s].name)),
                None => tr("Warrant: none", "Orden: ninguna").to_string(),
            },
        ),
        said(Tone::Plain, ""),
    ];
    lines.extend(game.log.iter().cloned());
    ui::log_rows(&f.body, &lines, width)
}

fn said(tone: Tone, text: &str) -> Line {
    Line { tone, speaker: None, text: text.to_string() }
}
