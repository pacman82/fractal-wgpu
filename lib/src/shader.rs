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

/// Number of iterations is bound as a Uniform variable available in the fragment shader stage. It
/// is the number of iterations we calculate of the complex sequence before we consider it
/// convergent.
pub const ITERATIONS_LAYOUT: BindGroupLayoutDescriptor = BindGroupLayoutDescriptor {
    label: Some("ITERATIONS BIND GROUP LAYOUT"),
    entries: &[BindGroupLayoutEntry {
        // Must match shader index
        binding: 0,
        // We only need this in the vertex shader
        visibility: ShaderStages::FRAGMENT,
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

/// Inverse view matrix padded to a multitude of 16bytes for compatibility with webGL.
pub fn inv_view_to_bytes(inv_view: &[[f32;2]; 3]) -> [u8; 64] {
    // Only way to reliable get the matrix to the shader for webGL is to put it into a 4x4 matrix.
    // There should be other ways, but empirically this is had been the only one working for me

    // Original 3x2 matrix layout
    // [ 1/z  0   tx]    | x |   | x/z + tx |
    // [  0  1/z  ty]  x | y | = | y/z - ty |
    //                   | 1 |
    // [
    //     [1/z, 0.],
    //     [0., 1/z],
    //     [x, y],
    // ]
    // Translated layout
    // [ 1/z  0  0  tx]    | x |   | x/z + tx |
    // [  0  1/z 0  ty]  x | y | = | y/z - ty |
    // [  0   0  0  0 ]  x | 0 | = |     0    |
    // [  0   0  0  0 ]  x | 1 | = |     0    |

    let four_by_four = [
        [inv_view[0][0], inv_view[0][1], 0., 0.],
        [inv_view[1][0], inv_view[1][1], 0., 0.],
        [0., 0., 0., 0.],
        [inv_view[2][0], inv_view[2][1], 0., 0.],
    ];



    let mut bytes = [0; 64];
    bytes.copy_from_slice(bytemuck::cast_slice(&four_by_four));
    bytes
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
        contents: inv_view_to_bytes(&init).as_slice(),
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

pub fn iterations_uniform(
    device: &Device,
    iterations: i32,
) -> (BindGroupLayout, Buffer, BindGroup) {
    let mut iterations_padded = [0; 4];
    iterations_padded[0] = iterations;

    let layout = device.create_bind_group_layout(&ITERATIONS_LAYOUT);
    let buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Iterations Buffer"),
        contents: bytemuck::cast_slice(&iterations_padded),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });
    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("Iterations Bind Group"),
        layout: &layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });
    (layout, buffer, bind_group)
}
