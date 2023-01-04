struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) coords: vec2<f32>
};

@vertex
fn vs_main(
    plane: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(plane.position, 0.0, 1.0);
    out.coords = plane.position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Find out how quickly the position in the complex plane
    // diverges.
    let c = in.coords;
    var z = vec2<f32>(0.0, 0.0);
    var i = 0;
    let iter = 500;
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
    let conv = f32(i) / f32(iter);

    var red: f32;
    var blue: f32;
    if i == 0 {
        red = 0.0;
        blue = 0.0;
    } else {
        red = 1.0 - conv;
        blue = conv;
    }
    let green = 0.0;

    let color = vec4<f32>(red, green, blue, 1.0);
    return color;
}