use wgpu::{
    BindGroup, BlendState, ColorTargetState, ColorWrites, Device, FragmentState, MultisampleState,
    PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, TextureFormat, VertexState, BufferUsages, util::{BufferInitDescriptor, DeviceExt}, Buffer,
};

use crate::{
    shader::{inv_view_uniform, CANVAS_SHADER_SOURCE, Vertex},
};

/// A specialised render pipeline for our 2D canvas.
///
/// Handles binding of vertices and inverse view matrix, loading shaders and binding their correct
/// input buffers to them.
pub struct CanvasRenderPipeline {
    render_pipeline: RenderPipeline,
    /// Used to pass the coordinates of the canvas to the shader in each render pass.
    vertex_buffer: Buffer,
    /// Used to pass the matrix to the vertex shader in each render pass.
    inv_view_bind_group: BindGroup,
}

impl CanvasRenderPipeline {
    pub fn new(
        device: &Device,
        surface_format: TextureFormat,
        initial_inv_view: [[f32; 2]; 3],
    ) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Canvas Shader"),
            source: ShaderSource::Wgsl(CANVAS_SHADER_SOURCE.into()),
        });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Canvas vertices"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });

        let (inv_view_layout, _inv_view_buffer, inv_view_bind_group) =
            inv_view_uniform(device, initial_inv_view);

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&inv_view_layout],
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
            vertex_buffer,
            inv_view_bind_group,
        }
    }

    pub fn draw<'s, 'p>(&'s self, render_pass: &mut RenderPass<'p>)
    where
        's: 'p,
    {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.inv_view_bind_group, &[]);
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