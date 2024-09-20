struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: u32,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    out.color = model.color;

    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let r = f32(in.color & 255) / 255.0;
    let g = f32(in.color >> 8 & 255) / 255.0;
    let b = f32(in.color >> 16 & 255) / 255.0;
    let a = f32(in.color >> 24 & 255) / 255.0;

    return  vec4<f32>(r, g, b, a);
}
