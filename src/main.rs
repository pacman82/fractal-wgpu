use anyhow::{Context, Error};
use log::error;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use self::canvas::Canvas;

mod canvas;

const WIDTH: u32 = 400;
const HEIGHT: u32 = 300;

fn main() -> Result<(), Error> {
    // We need logger to see wgpu error output
    env_logger::init();

    // WGP offers async function calls, pollster is a minimal async runtime
    pollster::block_on(run())
}

async fn run() -> Result<(), Error> {
    // Window message loop.
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Hello WGPU")
        .with_inner_size(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)))
        .build(&event_loop)?;

    let mut canvas = unsafe {
        Canvas::new(WIDTH, HEIGHT, &window)
            .await
            .context("Error requesting device for drawing")?
    };
    canvas.configure_surface();

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
            match canvas.render() {
                Ok(_) => (),
                // Most errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => error!("{e}"),
            }
        }
        _ => (),
    });
}
