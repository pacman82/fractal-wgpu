/// Inverse view matrix with padding so its size is a multitude of 16 Bytes. This is required for
/// running this shader with WebGL
struct VertexArgs {
    inv_view: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> VERTEX_ARGS: VertexArgs;

/// Uniform arguments for fragment shader, padedd to 16Bytes alignment for wegGL compatibility
struct FragmentArgs {
    iterations: i32,
    padding_0: i32,
    padding_1: i32,
    padding_2: i32,
}

@group(1) @binding(0)
var<uniform> FRAGMENT_ARGS: FragmentArgs;

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) coords: vec2<f32>,
};

@vertex
fn vs_main(
    plane: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(plane.position, 0.0, 1.0);
    // let inv_view = mat3x2(1.0, 0.0, 0.0, 1.0, -0.5, 0.0);
    out.coords = (VERTEX_ARGS.inv_view * vec4<f32>(plane.position, 0.0, 1.0)).xy;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Find out how quickly the position in the complex plane
    // diverges.
    let c = in.coords;
    var z = vec2<f32>(0.0, 0.0);
    var i = 0;
    let iter = FRAGMENT_ARGS.iterations;
    for (i=iter; i != 0; i--){
        let real = z.x * z.x - z.y * z.y + c.x;
        let imag = 2.0 * z.x * z.y + c.y;

        // Sequences with abs(z) > 2 will always diverge
        if (real * real + imag * imag > 4.0) {
            break;
        }

        z.x = real;
        z.y = imag;
    }
    let divergence = f32(i) / f32(iter);

    // if i == 0 {
    //     return vec4<f32> (0.,0.,0.,1.);
    // }

    // Most convergent colors first
    let colors = array(
        vec4<f32>(0.,0.,0.,1.),
        vec4<f32>(0.,1.,0.,1.),
        vec4<f32>(1.,0.,0.,1.),
        vec4<f32>(0.,0.,1.,1.),
    );
    // First half go into the first blend
    let end_first_blend = iter / 2; // The last color also gets the remainder
    let end_second_blend = iter / 4 + end_first_blend; // The last color also gets the remainder
    var first_color = vec4(0.,0.,0.,0.);
    var second_color = vec4(0.,0.,0.,0.);
    var blend = 0.0;
    if (i < end_first_blend) {
        first_color = colors[0];
        second_color = colors[1];
        blend = f32(i) / f32(end_first_blend);
    } else if (i < end_second_blend) {
        first_color = colors[1];
        second_color = colors[2];
        blend = f32(i - end_first_blend) / f32(end_second_blend - end_first_blend);
    } else {
        first_color = colors[2];
        second_color = colors[3];
        blend = f32(i - end_second_blend) / f32(iter - end_second_blend);
    }
    return (1. - blend) * first_color + blend * second_color;
}