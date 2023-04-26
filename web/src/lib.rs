//! This module is supposed to contain the WASM interface for fractal wgpu.
#![cfg(target_arch = "wasm32")]
use fractal_wgpu_lib::{Camera, Canvas};
use log::error;
use wasm_bindgen::prelude::wasm_bindgen;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::web::WindowExtWebSys,
    window::WindowBuilder,
};
const WIDTH: u32 = 400;
const HEIGHT: u32 = 400;

#[wasm_bindgen(start)]
pub async fn start() {
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

    let mut canvas = unsafe {
        Canvas::new(WIDTH, HEIGHT, &window)
            .await
            .expect("Error requesting device for drawing")
    };

    // Camera position and zoom level. Determines which part of the fractal we see
    let camera = Camera::new();
    // Number of iterations used to determine wether a point converges or not. How fast a point
    // converges is used to determine the color of a pixel.
    //
    // We use a floating point variable to track the number of iterations, so we can easier adapt
    // the number of iterations smoothly by pressing buttons for a period of time. This implies we
    // need to keep track of differences smaller than 1 between frames.
    let iterations = 256f32;

    match canvas.render(&camera, iterations.trunc() as i32) {
        Ok(_) => (),
        // Most errors (Outdated, Timeout) should be resolved by the next frame
        Err(e) => error!("Could not render frame: {e}"),
    }

    event_loop.run(move |event, _target, control_flow| match event {
        Event::WindowEvent {
            window_id: _,
            event: WindowEvent::CloseRequested,
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            window_id: _,
            event: WindowEvent::Resized(physical_size),
        } => {
            canvas.resize(physical_size.width, physical_size.height);
        }
        Event::WindowEvent {
            window_id: _,
            event:
                WindowEvent::ScaleFactorChanged {
                    scale_factor: _,
                    new_inner_size,
                },
        } => {
            canvas.resize(new_inner_size.width, new_inner_size.height);
        }
        Event::RedrawRequested(_window_id) => {
            match canvas.render(&camera, iterations.trunc() as i32) {
                Ok(_) => (),
                // Most errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => error!("Could not render frame: {e}"),
            }
        }
        Event::MainEventsCleared => {
            window.request_redraw();
            *control_flow = ControlFlow::Wait;
        }
        _ => (),
    });
}
