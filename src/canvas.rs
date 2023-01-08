use std::iter::once;

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferBindingType, BufferUsages, Color,
    ColorTargetState, ColorWrites, CommandEncoderDescriptor, CompositeAlphaMode, Device,
    DeviceDescriptor, Features, FragmentState, Limits, MultisampleState, Operations,
    PipelineLayoutDescriptor, PresentMode, PrimitiveState, PrimitiveTopology, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, RequestDeviceError, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    Surface, SurfaceConfiguration, SurfaceError, TextureFormat, TextureUsages,
    TextureViewDescriptor, VertexState,
};
use winit::window::Window;

use crate::{
    camera::Camera,
    vertex::{Vertex, VERTICES},
};

pub struct Canvas {
    /// Width of output surface in pixels.
    width: u32,
    /// Height of output surface in pixels.
    height: u32,
    camera: Camera,
    /// The surface we are rendering to. It is linked to the inner part of the window passed in the
    /// constructor.
    surface: Surface,
    /// The format of the texture. It is acquired using the preferred format of the adapter and we
    /// remember it, so we can recreate the surface if it becomes invalid.
    format: TextureFormat,
    /// A device is used to create buffers (for exchanging data with the GPU) among other things.
    device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    inv_view_buffer: Buffer,
    inv_view_bind_group: BindGroup,
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
        let inv_view_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Inverse view matrix"),
            contents: bytemuck::cast_slice(&[camera.inv_view()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let inv_view_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Inverse View Bind Group Layout"),
                entries: &[BindGroupLayoutEntry {
                    // Must match shader index
                    binding: 0,
                    // We only need this in the vertex shader
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        // All vertices see the same matrix
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let inv_view_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Inverse View Matrix Bind Group"),
            layout: &inv_view_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: inv_view_buffer.as_entire_binding(),
            }],
        });

        // Create render pipeline
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&inv_view_bind_group_layout],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::DESC],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multiview: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Canvas vertices"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });

        let canvas = Self {
            width,
            height,
            surface,
            device,
            queue,
            format,
            render_pipeline,
            vertex_buffer,
            inv_view_buffer,
            inv_view_bind_group,
            camera,
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
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.inv_view_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..(VERTICES.len() as u32), 0..1);
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
