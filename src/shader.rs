use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device,
    ShaderStages, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
};

/// Source used to compile the shader code at startup
pub const CANVAS_SHADER_SOURCE: &str = include_str!("shader.wgsl");

/// Inverse View matrix is bound as a Uniform variable available in the vertex shader stage. The
/// inverse view matrix is used to control which part of the canvas the user can see.
const INV_VIEW_LAYOUT: BindGroupLayoutDescriptor = BindGroupLayoutDescriptor {
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
};

/// Vertex as used in the vertex buffer of our canvas shader.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
}

impl Vertex {
    pub const DESC: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: size_of::<Self>() as u64,
        step_mode: VertexStepMode::Vertex,
        attributes: &[VertexAttribute {
            format: VertexFormat::Float32x2,
            offset: 0,
            shader_location: 0,
        }],
    };
}

/// The inverse view matrix is used to control which part of the canvas the user can see. This
/// return the layout, buffer and bindgroup for the inverse view matrix in one go.
pub fn inv_view_uniform(
    device: &Device,
    init: [[f32; 2]; 3],
) -> (BindGroupLayout, Buffer, BindGroup) {
    let layout = device.create_bind_group_layout(&INV_VIEW_LAYOUT);
    let buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Inverse view matrix"),
        contents: bytemuck::cast_slice(&[init]),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });
    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("Inverse View Matrix Bind Group"),
        layout: &layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });
    (layout, buffer, bind_group)
}
