use std::iter::once;

use anyhow::{Context, Error};
use log::error;
use wgpu::{
    Backends, CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, Features,
    Limits, PresentMode, Queue, RequestAdapterOptions, RequestDeviceError, Surface,
    SurfaceConfiguration, SurfaceError, TextureFormat, TextureUsages, TextureViewDescriptor,
};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

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
        .with_title("Hello Vulkan")
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
                // Surface Lost => Reconfigure surface
                Err(SurfaceError::Lost) => todo!(),
                // The system is out of memory, we should probably quit
                Err(SurfaceError::OutOfMemory) => {
                    error!("Not enough memory to allocate Frame.");
                    *control_flow = ControlFlow::Exit;
                }
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => error!("{e}"),
            }
        }
        _ => (),
    });
}

struct Canvas {
    width: u32,
    height: u32,
    surface: Surface,
    device: Device,
    queue: Queue,
    format: TextureFormat,
}

impl Canvas {
    async unsafe fn new(
        width: u32,
        height: u32,
        window: &Window,
    ) -> Result<Self, RequestDeviceError> {
        let instance = wgpu::Instance::new(Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        // Can be used for API call tracing if that feature is enabled.
        let trace_path = None;
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    features: Features::empty(),
                    limits: Limits::default(),
                },
                trace_path,
            )
            .await?;
        // The first format in the array is the prefered one.
        let format = surface.get_supported_formats(&adapter)[0];
        Ok(Self {
            width,
            height,
            surface,
            device,
            queue,
            format,
        })
    }

    /// Resize canvas to new size in pixels. Ignored if either width or height is zero.
    fn resize(&mut self, width: u32, height: u32) {
        // May be resized to an empty surface in case window is minimized. This would crash the
        // application, so we ignore resizing to an empty texture.
        if width != 0 || height != 0 {
            self.width = width;
            self.height = height;
            self.configure_surface();
        }
    }

    fn render(&self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        self.queue.submit(once(encoder.finish()));
        output.present();
        Ok(())
    }

    fn configure_surface(&self) {
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: self.format,
            width: self.width,
            height: self.height,
            present_mode: PresentMode::AutoVsync,
            alpha_mode: CompositeAlphaMode::Opaque,
        };
        self.surface.configure(&self.device, &config)
    }
}
