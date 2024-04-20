const CI_COMBINED_COLOR = 0;
const CI_TEXEL0_COLOR = 1;
const CI_TEXEL1_COLOR = 2;
const CI_PRIM_COLOR = 3;
const CI_SHADE_COLOR = 4;
const CI_ENV_COLOR = 5;
const CI_KEY_CENTER = 6;
const CI_KEY_SCALE = 7;
const CI_COMBINED_ALPHA = 8;
const CI_TEXEL0_ALPHA = 9;
const CI_TEXEL1_ALPHA = 10;
const CI_PRIM_ALPHA = 11;
const CI_SHADE_ALPHA = 12;
const CI_ENV_ALPHA = 13;
const CI_LOD_FRACTION = 14;
const CI_PRIM_LOD_FRACTION = 15;
const CI_NOISE = 16;
const CI_CONVERT_K4 = 17;
const CI_CONVERT_K5 = 18;
const CI_CONSTANT_1 = 19;
const CI_CONSTANT_0 = 20;

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

struct Constants {
    combine_rgb_0: u32,
    combine_rgb_1: u32,
    combine_alpha_0: u32,
    combine_alpha_1: u32,
    blend_0: u32,
    blend_1: u32,
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@group(2) @binding(0)
var<uniform> constants: Constants;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // TODO: Handle W coordinate
    let size = textureDimensions(t_diffuse);
    let s = in.tex_coords[0] / f32(size[0]);
    let t = in.tex_coords[1] / f32(size[1]);
    let sample = textureSample(t_diffuse, s_diffuse, vec2<f32>(s, t));
    return sample + (in.color / 255.0);
}