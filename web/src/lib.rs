//! This module is to contains the WASM interface for fractal wgpu.
#![cfg(target_arch = "wasm32")]
use fractal_wgpu_lib::{Camera, Canvas};
use log::error;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast};
use web_sys::HtmlCanvasElement;
use wgpu::SurfaceTarget;
use winit::{
    dpi::PhysicalSize,
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

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)))
        .build(&event_loop)
        .unwrap();

    let document = web_sys::window().unwrap().document().unwrap();
    let div = document.get_element_by_id("fractal-canvas").unwrap();
    let canvas = web_sys::Element::from(window.canvas().unwrap());
    let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>().unwrap();
    div.append_child(&canvas).unwrap();

    let surface_target = SurfaceTarget::Canvas(canvas);

    let mut canvas = Canvas::new(WIDTH, HEIGHT, surface_target)
        .await
        .expect("Error requesting device for drawing");

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

    event_loop
        .run(move |event, target| match event {
            Event::WindowEvent {
                window_id: _,
                event: WindowEvent::CloseRequested,
            } => {
                target.exit();
            }
            Event::WindowEvent {
                window_id: _,
                event: WindowEvent::Resized(physical_size),
            } => {
                canvas.resize(physical_size.width, physical_size.height);
            }
            Event::WindowEvent {
                window_id: _,
                event: WindowEvent::RedrawRequested,
            } => {
                match canvas.render(&camera, iterations.trunc() as i32) {
                    Ok(_) => (),
                    // Most errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => error!("Could not render frame: {e}"),
                }
            }
            Event::NewEvents(_) => {
                window.request_redraw();
                target.set_control_flow(ControlFlow::Wait);
            }
            _ => (),
        })
        .unwrap();
}
