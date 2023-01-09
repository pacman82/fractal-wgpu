use anyhow::{Context, Error};
use camera::Camera;
use log::error;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use self::canvas::Canvas;

mod camera;
mod canvas;
mod canvas_render_pipeline;
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
        .with_title("Hello WGPU")
        .with_inner_size(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)))
        .build(&event_loop)?;

    let mut canvas = unsafe {
        Canvas::new(WIDTH, HEIGHT, &window)
            .await
            .context("Error requesting device for drawing")?
    };

    let mut camera = Camera::new();

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
                    input: KeyboardInput {
                        scancode: _,
                        state: ElementState::Pressed,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                    is_synthetic: _,
                },
        } => {
            change_camera(&mut camera, keycode);
            window.request_redraw();
        }
        Event::RedrawRequested(_window_id) => {
            match canvas.render(&camera) {
                Ok(_) => (),
                // Most errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => error!("{e}"),
            }
        }
        _ => (),
    });
}

fn change_camera(camera: &mut Camera, keycode: VirtualKeyCode) {
    // Step size
    let s = 0.1;
    match keycode {
        VirtualKeyCode::Left => camera.change_pos(-s, 0.),
        VirtualKeyCode::Up => camera.change_pos(0., s),
        VirtualKeyCode::Right => camera.change_pos(s, 0.),
        VirtualKeyCode::Down => camera.change_pos(0., -s),
        // VirtualKeyCode::Back => (),
        VirtualKeyCode::Period => camera.zoom(1.02),
        VirtualKeyCode::Comma => camera.zoom(1. / 1.02),
        _ => ()
    };

}