//! The tests of the C++ side of the GUI: the window, the canvas and the
//! scene items, with input from Qt Test (mouse clicks, key presses).
//!
//! The tests are in `src/gui/gui_tests.cpp`. They run the real window with
//! the Qt `offscreen` platform, so no window shows on the screen and the
//! keyboard and the mouse of the user stay free.
//!
//! Qt must make its application in the main thread, so this file has its
//! own `main` (`harness = false` in `Cargo.toml`). Run it with
//! `cargo test --features gui-tests --test gui_offscreen`.

fn main() {
    std::process::exit(hdl_compose::gui::run_tests());
}
