//! This module is supposed to contain the WASM interface for fractal wgpu.

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub fn start() {
    // Show panics in web logging console
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
}