//! This module is to contains the WASM interface for fractal wgpu.
#![cfg(target_arch = "wasm32")]
use fractal_wgpu_lib::{Camera, Canvas};
use log::error;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast};
use web_sys::HtmlCanvasElement;
use wgpu::SurfaceTarget;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{Event, StartCause, WindowEvent},
    event_loop::{self, ActiveEventLoop, ControlFlow, EventLoop},
    platform::web::WindowExtWebSys,
    window::{Window, WindowId},
};
const WIDTH: u32 = 400;
const HEIGHT: u32 = 400;

struct App<'w> {
    canvas: Canvas<'w>,
    // Camera position and zoom level. Determines which part of the fractal we see
    camera: Camera,
    // Number of iterations used to determine wether a point converges or not. How fast a point
    // converges is used to determine the color of a pixel.
    //
    // We use a floating point variable to track the number of iterations, so we can easier adapt
    // the number of iterations smoothly by pressing buttons for a period of time. This implies we
    // need to keep track of differences smaller than 1 between frames.
    iterations: f32,
}

impl<'w> App<'w> {
    pub fn new(canvas: Canvas<'w>) -> Self {
        let camera = Camera::new();
        let iterations = 256f32;
        Self {
            canvas,
            camera,
            iterations,
        }
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                self.canvas
                    .resize(physical_size.width, physical_size.height);
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor: _,
                inner_size_writer: _,
            } => {
                // We use mathematically cordinates for camera position rather than pixels, so we
                // are fine without explicitly handling scale factor changes.
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                is_synthetic: _,
                event,
            } => {
                // self.controls.track_button_presses(event);
            }
            WindowEvent::RedrawRequested => {
                // self.redraw_requested = true;
            }
            _ => (),
        }
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, _cause: StartCause) {
        event_loop.set_control_flow(ControlFlow::Wait);
    }
}

#[wasm_bindgen(start)]
pub async fn start() {
    // Show panics in web logging console
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");

    let event_loop = EventLoop::new().unwrap();
    let window = event_loop
        .create_window(
            Window::default_attributes()
                .with_inner_size(PhysicalSize::new(f64::from(WIDTH), f64::from(HEIGHT))),
        )
        .unwrap();

    let document = web_sys::window().unwrap().document().unwrap();
    let div = document.get_element_by_id("fractal-canvas").unwrap();
    let canvas = web_sys::Element::from(window.canvas().unwrap());
    let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>().unwrap();
    div.append_child(&canvas).unwrap();

    let surface_target = SurfaceTarget::Canvas(canvas);

    let canvas = Canvas::new(WIDTH, HEIGHT, surface_target)
        .await
        .expect("Error requesting device for drawing");

    let mut app = App::new(canvas);

    match app
        .canvas
        .render(&app.camera, app.iterations.trunc() as i32)
    {
        Ok(_) => (),
        // Most errors (Outdated, Timeout) should be resolved by the next frame
        Err(e) => error!("Could not render frame: {e}"),
    }

    event_loop.run_app(&mut app).unwrap();
}
