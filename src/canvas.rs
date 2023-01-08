use std::iter::once;

use wgpu::{
    Backends, Color, CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor,
    Features, Limits, Operations, PresentMode, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RequestAdapterOptions, RequestDeviceError, Surface, SurfaceConfiguration,
    SurfaceError, TextureFormat, TextureUsages, TextureViewDescriptor,
};
use winit::window::Window;

use crate::{camera::Camera, canvas_render_pipeline::CanvasRenderPipeline};

pub struct Canvas {
    /// Width of output surface in pixels.
    width: u32,
    /// Height of output surface in pixels.
    height: u32,
    /// The surface we are rendering to. It is linked to the inner part of the window passed in the
    /// constructor.
    surface: Surface,
    /// The format of the texture. It is acquired using the preferred format of the adapter and we
    /// remember it, so we can recreate the surface if it becomes invalid.
    format: TextureFormat,
    /// A device is used to create buffers (for exchanging data with the GPU) among other things.
    device: Device,
    queue: Queue,
    render_pipeline: CanvasRenderPipeline,
}

impl Canvas {
    /// Construct a new canvas and link it to a window. Height and width are specified in pixels.
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

        // Inverse view Matrix
        let camera = Camera::new();

        let render_pipeline = CanvasRenderPipeline::new(&device, format, camera.inv_view());

        let canvas = Self {
            width,
            height,
            surface,
            device,
            queue,
            format,
            render_pipeline,
        };
        canvas.configure_surface();

        Ok(canvas)
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
            Err(other) => return Err(other),
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
                    load: wgpu::LoadOp::Clear(Color {
                        r: 0.3,
                        g: 0.2,
                        b: 0.7,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        };
        {
            let mut render_pass = encoder.begin_render_pass(&rpd);
            self.render_pipeline.draw(&mut render_pass);
        }
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
