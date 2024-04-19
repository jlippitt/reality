struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coords: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> scissor: vec4<f32>;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let x = ((in.position[0] - scissor[0]) * 2.0 / scissor[2]) - 1.0;
    let y = 1.0 - ((in.position[1] - scissor[1]) * 2.0 / scissor[3]);
    let z = (in.position[2] / 65536.0) + 0.5;
    out.clip_position = vec4<f32>(x, y, z, 1.0);
    out.color = in.color;
    out.tex_coords = in.tex_coords;
    return out;
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // TODO: Handle W coordinate
    let size = textureDimensions(t_diffuse);
    let s = in.tex_coords[0] / f32(size[0]);
    let t = in.tex_coords[1] / f32(size[1]);
    let sample = textureSample(t_diffuse, s_diffuse, vec2<f32>(s, t));
    return sample + (in.color / 255.0);
}