# carmen-esp32

Where in the World? on an ESP32-S3: a standalone, touch-screen version of
[Carmen](https://github.com/jonorl/carmen), the family-edition remake of
*Where in the World is Carmen Sandiego?*. A separate project: it copies what
it needs from the game rather than sharing code with it.

It's built for an ESP32-S3 DevKitC-1 N16R8 (16 MB flash, 8 MB octal PSRAM)
with a 2.8" 320x240 ILI9341 SPI screen with an XPT2046 resistive touch panel,
and a passive piezo (Murata PKM22EPPH4001-B, loudest near 4 kHz).

So far it shows a test pattern, then each country's postcard in turn at each
time of day, logging how long each takes to paint and send.

## Wiring

| Module pin | ESP32-S3 | Note |
|---|---|---|
| VCC | 5V | through the module's regulator; leave J1 open |
| GND | GND | |
| CS | GPIO 10 | |
| RESET | GPIO 8 | |
| DC | GPIO 9 | |
| SDI (MOSI) | GPIO 11 | |
| SCK | GPIO 12 | |
| LED | 3V3 | backlight, always on |
| SDO (MISO) | not connected | never read, and on these modules it can hold the shared line against the touch chip |
| T_CLK | GPIO 12 | shared with SCK |
| T_CS | GPIO 7 | |
| T_DIN | GPIO 11 | shared with SDI |
| T_DO | GPIO 13 | |
| T_IRQ | GPIO 6 | |
| Piezo | GPIO 4 and GND | either way round; GND leg to GPIO 5 to drive it from both sides if too quiet |

GPIO 10-13 are SPI2's own pins, so the screen can run at 40 MHz. Kept clear:
0, 3, 45 and 46 (boot straps), 19 and 20 (USB), 26-37 (flash and PSRAM), 43
and 44 (the console) and the RGB LED (38 or 48).

## Building

Needs the `esp` toolchain (`espup install`), `ldproxy` and `espflash`. The
first build downloads ESP-IDF. Plug the board in by its USB port, then in this
folder:

```
cargo run --release
```

builds, flashes (with `partitions.csv`, an 8 MB app partition) and opens the
serial monitor.

## Copied from Carmen

Copied from the game at commit `4e0a45c`, then freed of macroquad:

- `src/art/pixels.rs`, `scene.rs`, `landmarks.rs`: the postcards
  (`scene::paint` gives a `Canvas`).
- `src/rules/game.rs`, `data.rs`, `rng.rs`: the rules and data loading.
  Changed here: `Game::route` is public, for the self-playing check, and one
  `rng.choice` call names its type (`::<&str>`), which the `esp` toolchain's
  newer Rust can't infer.
- `src/lang.rs`: English and Spanish. Unchanged: the device has no system
  language, so it starts in English until one is chosen; the choice is saved
  through `src/device/storage.rs` (flash) instead of the game's files.
- `countries.json`, `suspects.json`
- `assets/fonts/`: Pixelify Sans, VT323 and Noto Sans (₹ only), already cut
  down to the characters the game shows, with their SIL OFL licences.

Changes to these in the game don't arrive here by themselves: carry them over
by hand, and note the new commit here. Their `#[cfg(test)]` tests don't run
here (there's no test harness on the device); run them in the game.

## Checking the wiring

```
cargo run --release --bin pins
```

flashes a wiring check instead (`src/bin/pins.rs`) and prints its findings:
pins stuck to GND or a supply, pins joined to each other, whether touching
the screen reaches T_IRQ, and whether the display chip answers (join the
display's SDO to GPIO 13 for that part). Then each pin blinks in turn (once a
second for 6 s), for a meter or an LED on the display's pins.

`cargo run --release --bin screen` (`src/bin/screen.rs`) shows the `mipidsi`
crate's test image, then the whole screen red, green, blue and white. A first
driver of our own drew corrupted pictures where this one drew right, so
`src/display.rs` now draws through `mipidsi` too.

`cargo run --release --bin case` (`src/bin/case.rs`) runs the game's rules
with nothing drawn: it loads and checks the data, then plays a case in English
and one in Spanish as a careful player would, printing everything the game
says with timings and free memory. It also counts its starts in flash.

`cargo run --release --bin text` (`src/bin/text.rs`) shows the fonts with
the game's own text in both languages: body text in Pixelify Sans at 13 px,
smoothed (chosen on the screen over 11 and 14 px), across the screen and in
a half-screen panel; labels at 11 px and headings at 22 px, both the font's
own pixels; and VT323 at 25 px for the printer.

`cargo run --release --bin touch` (`src/bin/touch.rs`) tests the touch panel
(`src/touch.rs`): tap the three crosshairs to calibrate it, then draw with a
finger; the red box at the top right clears the screen.
