const CI_COMBINED_COLOR: u32 = 0;
const CI_TEXEL0_COLOR: u32 = 1;
const CI_TEXEL1_COLOR: u32 = 2;
const CI_PRIM_COLOR: u32 = 3;
const CI_SHADE_COLOR: u32 = 4;
const CI_ENV_COLOR: u32 = 5;
const CI_KEY_CENTER: u32 = 6;
const CI_KEY_SCALE: u32 = 7;
const CI_COMBINED_ALPHA: u32 = 8;
const CI_TEXEL0_ALPHA: u32 = 9;
const CI_TEXEL1_ALPHA: u32 = 10;
const CI_PRIM_ALPHA: u32 = 11;
const CI_SHADE_ALPHA: u32 = 12;
const CI_ENV_ALPHA: u32 = 13;
const CI_LOD_FRACTION: u32 = 14;
const CI_PRIM_LOD_FRACTION: u32 = 15;
const CI_NOISE: u32 = 16;
const CI_CONVERT_K4: u32 = 17;
const CI_CONVERT_K5: u32 = 18;
const CI_CONSTANT_1: u32 = 19;
const CI_CONSTANT_0: u32 = 20;

const BI_COMBINED_COLOR: u32 = 0;
const BI_MEMORY_COLOR: u32 = 1;
const BI_BLEND_COLOR: u32 = 2;
const BI_FOG_COLOR: u32 = 3;

const BFA_COMBINED_ALPHA: u32 = 0;
const BFA_FOG_ALPHA: u32 = 1;
const BFA_SHADE_ALPHA: u32 = 2;
const BFA_CONSTANT_0: u32 = 3;

const BFB_ONE_MINUS_A: u32 = 0;
const BFB_MEMORY_ALPHA: u32 = 1;
const BFB_CONSTANT_1: u32 = 2;
const BFB_CONSTANT_0: u32 = 3;

const CT_ONE_CYCLE: u32 = 0;
const CT_TWO_CYCLE: u32 = 1;
const CT_COPY: u32 = 2;
const CT_FILL: u32 = 3;

struct Combine {
    sub_a: u32,
    sub_b: u32,
    mul: u32,
    add: u32,
}

struct Blend {
    p: u32,
    a: u32,
    m: u32,
    b: u32,
}

struct Constants {
    combine_rgb_0: Combine,
    combine_rgb_1: Combine,
    combine_alpha_0: Combine,
    combine_alpha_1: Combine,
    blend_0: Blend,
    blend_1: Blend,
    cycle_type: u32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coords: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec3<f32>,
    @location(2) fill_select: f32,
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
    out.fill_select = in.position[0];
    return out;
}

@group(1) @binding(0)
var t_fill_color: texture_2d<f32>;
@group(1) @binding(1)
var s_fill_color: sampler;

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var s_diffuse: sampler;

@group(3) @binding(0)
var<uniform> constants: Constants;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if constants.cycle_type == CT_FILL {
        return textureSample(t_fill_color, s_fill_color, vec2<f32>(in.fill_select / 4.0, 0.0));
    }

    // TODO: Handle W coordinate
    let size = textureDimensions(t_diffuse);
    let s = in.tex_coords[0] / f32(size[0]);
    let t = in.tex_coords[1] / f32(size[1]);
    let sample = textureSample(t_diffuse, s_diffuse, vec2<f32>(s, t));
    return sample + in.color;
}