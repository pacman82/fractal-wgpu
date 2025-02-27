use anyhow::{Context, Error};
use controls::Controls;
use log::error;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use fractal_wgpu_lib::{Camera, Canvas};

mod controls;

const WIDTH: u32 = 400;
const HEIGHT: u32 = 400;

const GREETING: &str = include_str!("greeting.txt");

fn main() -> Result<(), Error> {
    // We need logger to see wgpu error output
    env_logger::init();

    println!("{GREETING}");

    // WGP offers async function calls, pollster is a minimal async runtime
    pollster::block_on(run())
}

struct App<'w> {
    /// Keeps track of request redraw request, e.g if the window has been partially hidden behind
    /// another window, ro is resized.
    redraw_requested: bool,
    /// We can only initialize the canvas after the window is created in `resumed`.
    canvas: Canvas<'w>,
    /// Number of iterations used to determine wether a point converges or not. How fast a point
    /// converges is used to determine the color of a pixel.
    ///
    /// We use a floating point variable to track the number of iterations, so we can easier adapt
    /// the number of iterations smoothly by pressing buttons for a period of time. This implies we
    /// need to keep track of differences smaller than 1 between frames.
    iterations: f32,
    // Camera position and zoom level. Determines which part of the fractal we see
    camera: Camera,
    controls: Controls,
}

impl<'w> App<'w> {
    async fn new(window: &'w Window) -> Result<Self, Error> {
        let canvas = pollster::block_on(async move { Canvas::new(WIDTH, HEIGHT, window).await })
            .context("Error requesting device for drawing")
            .unwrap();
        Ok(Self {
            iterations: 256f32,
            redraw_requested: true,
            canvas,
            camera: Camera::new(),
            controls: Controls::new(),
        })
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {

    }

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
                self.controls.track_button_presses(event);
            }
            WindowEvent::RedrawRequested => {
                self.redraw_requested = true;
            }
            _ => (),
        }
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
        self.controls
            .update_scene(&mut self.camera, &mut self.iterations);
        if self.redraw_requested || self.controls.picture_changes() {
            match self
                .canvas
                .render(&self.camera, self.iterations.trunc() as i32)
            {
                Ok(_) => (),
                // Most errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => error!("{e}"),
            }
        }
        self.redraw_requested = false;
        // If the camera is not moving or zooming, we behave like a "normal" event driver window
        // app patiently waiting for the next event and not waisting CPU cycles in a busy loop.
        // Should we however change the picture we switch to polling as in a game loop, for
        // smooth control.
        if self.controls.picture_changes() {
            event_loop.set_control_flow(ControlFlow::Poll);
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}

async fn run() -> Result<(), Error> {
    // Window message loop.
    let event_loop = EventLoop::new().unwrap();
    let window = event_loop
        .create_window(
            Window::default_attributes()
                .with_title("Fractal WGPU")
                .with_inner_size(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT))),
        )
        .unwrap();
    let mut app = App::new(&window).await?;
    event_loop.run_app(&mut app).unwrap();
    Ok(())
}
