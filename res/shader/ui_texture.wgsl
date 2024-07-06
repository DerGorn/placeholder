struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) tex_index: u32,
    @location(3) blend_color: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tex_index: u32,
    @location(2) blend_color: u32,
};

struct CameraUniform {
    projection: mat3x2<f32>,
}
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.tex_index = model.tex_index;
    out.blend_color = model.blend_color;

    let position = camera.projection * vec3<f32>(model.position, 1.0);
    out.clip_position = vec4<f32>(position, 0.0, 1.0);

    return out;
}

// Fragment shader
@group(0) @binding(0) 
var texture_array: binding_array<texture_2d<f32>>;
@group(0) @binding(1) 
var sampler_array: binding_array<sampler>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var sample = textureSampleLevel(texture_array[in.tex_index], sampler_array[in.tex_index], in.tex_coords, 0.0);

    if in.blend_color != 0 {
        let alpha_b = sample.a;
        let alpha_a = f32(in.blend_color & 255) / 255.0;
        let b = f32(in.blend_color >> 8 & 255) / 255.0;
        let g = f32(in.blend_color >> 16 & 255) / 255.0;
        let r = f32(in.blend_color >> 24 & 255) / 255.0;
        sample = vec4<f32>(r, g, b, alpha_a);
        if alpha_b == 0.0 {
            sample.a = alpha_b;
        }
    } 
    return sample;
}
