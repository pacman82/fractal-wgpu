use anyhow::{Context, Error};
use camera::Camera;
use controls::Controls;
use log::error;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use self::canvas::Canvas;

mod camera;
mod canvas;
mod canvas_render_pipeline;
mod controls;
mod shader;

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

async fn run() -> Result<(), Error> {
    // Window message loop.
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Fractal WGPU")
        .with_inner_size(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)))
        .build(&event_loop)?;

    let mut canvas = unsafe {
        Canvas::new(WIDTH, HEIGHT, &window)
            .await
            .context("Error requesting device for drawing")?
    };

    // Keeps track of request redraw request, e.g if the window has been partially hidden behind
    // another window, ro is resized.
    let mut redraw_requested = true;
    // True if the picture we want to display on the canvas changed (e.g. due to a change in camera
    // position).
    let mut camera = Camera::new();
    let mut controls = Controls::new();

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
        Event::WindowEvent {
            window_id: _,
            event:
                WindowEvent::KeyboardInput {
                    device_id: _,
                    input,
                    is_synthetic: _,
                },
        } => {
            controls.track_button_presses(input);
        }
        Event::RedrawRequested(_window_id) => {
            redraw_requested = true;
        }
        Event::MainEventsCleared => {
            controls.change_camera(&mut camera);
            if redraw_requested || controls.picture_changes() {
                match canvas.render(&camera) {
                    Ok(_) => (),
                    // Most errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => error!("{e}"),
                }
            }
            redraw_requested = false;
            // If the camera is not moving or zooming, we behave like a "normal" event driver window
            // app patiently waiting for the next event and not waisting CPU cycles in a busy loop.
            // Should we however change the picture we switch to polling as in a game loop, for
            // smooth control.
            *control_flow = if controls.picture_changes() {
                ControlFlow::Poll
            } else {
                ControlFlow::Wait
            };
        }
        _ => (),
    });
}
