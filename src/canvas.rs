use std::iter::once;

use wgpu::{Surface, Device, Queue, TextureFormat, RequestDeviceError, Backends, RequestAdapterOptions, DeviceDescriptor, Features, Limits, SurfaceError, TextureViewDescriptor, CommandEncoderDescriptor, RenderPassDescriptor, RenderPassColorAttachment, Operations, Color, SurfaceConfiguration, TextureUsages, PresentMode, CompositeAlphaMode};
use winit::window::Window;

pub struct Canvas {
    width: u32,
    height: u32,
    surface: Surface,
    device: Device,
    queue: Queue,
    format: TextureFormat,
}

impl Canvas {
    pub async unsafe fn new(
        width: u32,
        height: u32,
        window: &Window,
    ) -> Result<Self, RequestDeviceError> {
        let instance = wgpu::Instance::new(Backends::DX12);
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
    pub fn resize(&mut self, width: u32, height: u32) {
        // May be resized to an empty surface in case window is minimized. This would crash the
        // application, so we ignore resizing to an empty texture.
        if width != 0 || height != 0 {
            self.width = width;
            self.height = height;
            self.configure_surface();
        }
    }

    pub fn render(&self) -> Result<(), SurfaceError> {
        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            // Surface Lost => Reconfigure surface
            Err(SurfaceError::Lost) => {
                self.configure_surface();
                self.surface.get_current_texture()?
            }
            Err(other) => {
                return Err(other)
            }
        };
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let rpd = RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Clear(Color { r: 0.3, g: 0.2, b: 0.7, a: 1.0 }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        };
        {
            let _render_pass = encoder.begin_render_pass(&rpd);
        }
        self.queue.submit(once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn configure_surface(&self) {
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