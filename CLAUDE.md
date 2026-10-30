# carmen-esp32

A standalone ESP32-S3 version of the Carmen game (`../carmen`, github.com/jonorl/carmen), played on a 320x240 ILI9341 touch screen with a piezo. A separate project: never change the game's repository from here. Files taken from the game are copies, listed with the commit they came from under "Copied from Carmen" in `README.md`; update that list when copying more.

Hardware, wiring and commands are in `README.md`.

## Files

- `src/lib.rs`: the library the programs share; `src/main.rs` and `src/bin/*` use it as `carmen_esp32::...`.
- `src/main.rs`: the demo so far (test pattern, then every postcard in turn, logging paint and send times).
- `src/display.rs`: the screen through the `mipidsi` crate, landscape (`Rotation::Deg90`), BGR, 40 MHz, no DMA. `blit` and `fill` take `[u8; 3]` colours.
- `src/touch.rs`: the XPT2046 on the same SPI bus (own chip select, 2 MHz). `Calibration::MEASURED` maps raw readings to screen pixels.
- `src/art/`, `src/rules/`, `src/lang.rs`, `countries.json`, `suspects.json`: copied from the game (see the README for what was changed in the copies).
- `src/device/storage.rs`: the game's `storage::read` and `write`, kept in NVS flash; `storage::start` must run first.
- `src/bin/`: checks, `pins` (wiring), `screen` (mipidsi's test image), `touch` (calibration and drawing), `case` (the rules playing a case, printed to the serial monitor).

## Learnt on the hardware

- A hand-written ILI9341 driver drew corrupted pictures on this module (orientation and colour settings ignored) where `mipidsi` drew it right on the same wiring. Draw through `mipidsi`; the cause in the hand-written one was never found.
- DMA on the SPI bus is untested since then: turn it on alone and check the pictures.
- T_IRQ goes low only now and then on presses, so touches are detected by pressure (`MIN_PRESSURE`), not by T_IRQ.
- The touch panel is turned 90 degrees to the picture: raw X runs down the screen and raw Y across it (`swap: true`).
- Taps need debouncing: one press read as two taps until a release had to last 0.3 s.
- The display runs from the board's 5V pin (USB power) on VCC with jumper J1 open, and its GND joined to the board's. The LED pin on 3V3 is already full brightness. The panel looks washed out off-axis.

## Commands

- `cargo run --release`: build, flash and monitor the demo.
- `cargo run --release --bin pins|screen|touch|case`: the checks.
- `cargo fmt`: 140 columns (`rustfmt.toml`).
