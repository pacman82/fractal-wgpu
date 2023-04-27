use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BlendState, Buffer, BufferUsages, Color, ColorTargetState, ColorWrites,
    CommandEncoder, Device, FragmentState, MultisampleState, Operations, PipelineLayoutDescriptor,
    PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, TextureFormat,
    TextureView, VertexState,
};

use crate::shader::{inv_view_uniform, iterations_uniform, Vertex, CANVAS_SHADER_SOURCE, inv_view_to_bytes};

/// A specialised render pipeline for our 2D canvas.
///
/// Handles binding of vertices and inverse view matrix, loading shaders and binding their correct
/// input buffers to them.
pub struct CanvasRenderPipeline {
    render_pipeline: RenderPipeline,
    /// Used to pass the coordinates of the canvas to the shader in each render pass.
    vertex_buffer: Buffer,
    /// We hold the buffer explicitly, so we can manipulate its contents between frames to change
    /// the camera positon.
    inv_view_buffer: Buffer,
    /// Used to pass the inverse view matrix in `inv_view_buffer` to the vertex shader in each
    /// render pass.
    inv_view_bind_group: BindGroup,
    /// We hold the buffer explicitly, so we can manipulate its contents between frames to how much
    /// elements of the sequence we calculate before we consider it convergent.
    iter_buffer: Buffer,
    /// Used to pass the number of iterations in `iter_buffer` to the fragment shader in each render
    /// pass.
    iter_bind_group: BindGroup,
}

impl CanvasRenderPipeline {
    /// Creates a new render pipeline for our canvas.
    ///
    /// # Parameters
    ///
    /// * `device` is used to create the render pipeline, load shaders and bind buffers.
    /// * `surface_format` is the format of the target (output) for the render pipeline.
    pub fn new(device: &Device, surface_format: TextureFormat) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Canvas Shader"),
            source: ShaderSource::Wgsl(CANVAS_SHADER_SOURCE.into()),
        });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Canvas vertices"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });

        let initial_inv_view = [[0., 0.]; 3];
        let (inv_view_layout, inv_view_buffer, inv_view_bind_group) =
            inv_view_uniform(device, initial_inv_view);

        let (iter_layout, iter_buffer, iter_group) = iterations_uniform(device, 1);

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&inv_view_layout, &iter_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Canvas Render Pipeline"),
            layout: Some(&layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::DESC],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: surface_format,
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

        CanvasRenderPipeline {
            render_pipeline,
            inv_view_buffer,
            vertex_buffer,
            inv_view_bind_group,
            iter_buffer,
            iter_bind_group: iter_group,
        }
    }

    /// Updates the buffers submitted to the shaders in each frame.
    pub fn update_buffers(&self, queue: &Queue, inv_view_matrix: [[f32; 2]; 3], iterations: i32) {
        queue.write_buffer(
            &self.inv_view_buffer,
            0,
            inv_view_to_bytes(&inv_view_matrix).as_slice()
        );
        let mut iterations_padded = [0i32; 4];
        iterations_padded[0] = iterations;
        queue.write_buffer(
            &self.iter_buffer,
            0,
            bytemuck::cast_slice(&iterations_padded),
        );
    }

    pub fn draw_to(&self, output: &TextureView, encoder: &mut CommandEncoder) {
        let rpd = RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: output,
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

        let mut render_pass = encoder.begin_render_pass(&rpd);
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.inv_view_bind_group, &[]);
        render_pass.set_bind_group(1, &self.iter_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..(VERTICES.len() as u32), 0..1);
    }
}

/// Rectangle vertex strip spanning the entire surface
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, 1.0],
    },
    Vertex {
        position: [-1.0, -1.0],
    },
    Vertex {
        position: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0],
    },
];
