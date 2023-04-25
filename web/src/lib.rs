//! This module is supposed to contain the WASM interface for fractal wgpu.
#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::wasm_bindgen;
use winit::{
    event_loop::EventLoop,
    platform::web::WindowExtWebSys,
    window::WindowBuilder, dpi::LogicalSize,
};

#[wasm_bindgen(start)]
pub fn start() {
    // Show panics in web logging console
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(f64::from(400), f64::from(400)))
        .build(&event_loop)
        .unwrap();

    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let dst = doc.get_element_by_id("fractal-canvas")?;
            let canvas = web_sys::Element::from(window.canvas());
            dst.append_child(&canvas).ok()?;
            Some(())
        })
        .expect("Couldn't append canvas to document body.");
}
