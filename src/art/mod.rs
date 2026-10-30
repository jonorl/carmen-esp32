//! Pictures, copied from the Carmen game's `src/art` (see the README) and
//! freed of macroquad: the device paints into a `Canvas` and sends its
//! pixels to the screen itself. `postcard` is the device's own: it fits the
//! game's postcards to the smaller picture frame.

pub mod landmarks;
pub mod pixels;
pub mod postcard;
pub mod scene;
pub mod sprites;
