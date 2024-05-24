struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) tex_index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tex_index: u32,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.tex_index = model.tex_index;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader
@group(0) @binding(0) 
var texture_array: binding_array<texture_2d<f32>>;
@group(0) @binding(1) 
var sampler_array: binding_array<sampler>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(textureSampleLevel(texture_array[in.tex_index], sampler_array[in.tex_index], in.tex_coords, 0.0).rgb, 1.0);
}
