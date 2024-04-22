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
    fog_color: vec4<f32>,
    blend_color: vec4<f32>,
    prim_color: vec4<f32>,
    env_color: vec4<f32>,
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
    out.clip_position = vec4<f32>(x, y, in.position[2], 1.0);
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

    if constants.cycle_type == CT_COPY {
        return vec4(sample[0], sample[1], sample[2], 1.0);
    }

    if constants.cycle_type == CT_ONE_CYCLE {
        let combined = vec4<f32>(
            combine_rgb(constants.combine_rgb_1, sample, in.color, vec4<f32>()),
            combine_alpha(constants.combine_alpha_1, sample[3], in.color[3], 0.0),
        );

        return blend(constants.blend_0, in.color[3], combined);
    }

    let combined_0 = vec4<f32>(
        combine_rgb(constants.combine_rgb_0, sample, in.color, vec4<f32>()),
        combine_alpha(constants.combine_alpha_0, sample[3], in.color[3], 0.0),
    );

    let combined_1 = vec4<f32>(
        combine_rgb(constants.combine_rgb_1, sample, in.color, combined_0),
        combine_alpha(constants.combine_alpha_1, sample[3], in.color[3], combined_0[3]),
    );

    let blended_0 = blend(constants.blend_0, in.color[3], combined_1);

    return blend(constants.blend_1, in.color[3], blended_0);
}

fn combine_rgb(combine: Combine, tex0: vec4<f32>, shade: vec4<f32>, combined: vec4<f32>) -> vec3<f32> {
    let sub_a = combine_rgb_input(combine.sub_a, tex0, shade, combined);
    let sub_b = combine_rgb_input(combine.sub_b, tex0, shade, combined);
    let mul = combine_rgb_input(combine.mul, tex0, shade, combined);
    let add = combine_rgb_input(combine.add, tex0, shade, combined);
    return (sub_a - sub_b) * mul + add;
}

fn combine_alpha(combine: Combine, tex0: f32, shade: f32, combined: f32) -> f32 {
    let sub_a = combine_alpha_input(combine.sub_a, tex0, shade, combined);
    let sub_b = combine_alpha_input(combine.sub_b, tex0, shade, combined);
    let mul = combine_alpha_input(combine.mul, tex0, shade, combined);
    let add = combine_alpha_input(combine.add, tex0, shade, combined);
    return (sub_a - sub_b) * mul + add;
}

fn blend(blend: Blend, shade_alpha: f32, combined: vec4<f32>) -> vec4<f32> {
    let color = vec3<f32>(combined[0], combined[1], combined[2]);

    let p = blend_input(blend.p, vec3<f32>(color));
    let a = blend_factor_a(blend.a, shade_alpha, combined[3]);

    if blend.m == BI_MEMORY_COLOR {
        // Alpha blending
        // TODO: B values other than 0
        return vec4<f32>(p, a);
    }

    // Shader blending
    // TODO: What to do when B=1?
    let m = blend_input(blend.m, vec3<f32>(color));
    let b = blend_factor_b(blend.b, a);

    return vec4<f32>(p * a + m * b, 1.0);
}

fn combine_rgb_input(input: u32, tex0: vec4<f32>, shade: vec4<f32>, combined: vec4<f32>) -> vec3<f32> {
    switch input {
        case CI_COMBINED_COLOR: { return vec3<f32>(combined[0], combined[1], combined[2]); }
        case CI_TEXEL0_COLOR: { return vec3<f32>(tex0[0], tex0[1], tex0[2]); }
        case CI_TEXEL1_COLOR: { return vec3<f32>(0.0); } // TODO
        case CI_PRIM_COLOR: {
            return vec3<f32>(
                constants.prim_color[0],
                constants.prim_color[1],
                constants.prim_color[2],
            );
        }
        case CI_SHADE_COLOR: { return vec3<f32>(shade[0], shade[1], shade[2]); }
        case CI_ENV_COLOR: {
            return vec3<f32>(
                constants.env_color[0],
                constants.env_color[1],
                constants.env_color[2],
            );
        }
        case CI_KEY_CENTER: { return vec3<f32>(0.0); } // TODO
        case CI_KEY_SCALE: { return vec3<f32>(0.0); } // TODO
        case CI_COMBINED_ALPHA: { return vec3<f32>(combined[3]); }
        case CI_TEXEL0_ALPHA: { return vec3<f32>(tex0[3]); }
        case CI_TEXEL1_ALPHA: { return vec3<f32>(0.0); } // TODO
        case CI_PRIM_ALPHA: { return vec3<f32>(constants.prim_color[3]); }
        case CI_SHADE_ALPHA: { return vec3<f32>(shade[3]); }
        case CI_ENV_ALPHA: { return vec3<f32>(constants.env_color[3]); }
        case CI_LOD_FRACTION: { return vec3<f32>(1.0); } // TODO
        case CI_PRIM_LOD_FRACTION: { return vec3<f32>(1.0); } // TODO
        case CI_NOISE: { return vec3<f32>(0.0); } // TODO
        case CI_CONVERT_K4: { return vec3<f32>(0.0); } // TODO
        case CI_CONVERT_K5: { return vec3<f32>(0.0); } // TODO
        case CI_CONSTANT_1: { return vec3<f32>(1.0); }
        case CI_CONSTANT_0: { return vec3<f32>(0.0); }
        default: { return vec3<f32>(0.0); }
    }
}

fn combine_alpha_input(input: u32, tex0: f32, shade: f32, combined: f32) -> f32 {
    switch input {
        case CI_COMBINED_ALPHA: { return combined; }
        case CI_TEXEL0_ALPHA: { return tex0; }
        case CI_TEXEL1_ALPHA: { return 0.0; } // TODO
        case CI_PRIM_ALPHA: { return constants.prim_color[3]; }
        case CI_SHADE_ALPHA: { return shade; }
        case CI_ENV_ALPHA: { return constants.env_color[3]; }
        case CI_LOD_FRACTION: { return 1.0; } // TODO
        case CI_PRIM_LOD_FRACTION: { return 1.0; } // TODO
        case CI_CONSTANT_1: { return 1.0; }
        case CI_CONSTANT_0: { return 0.0; }
        default: { return 0.0; }
    }
}

fn blend_input(input: u32, combined: vec3<f32>) -> vec3<f32> {
    switch input {
        case BI_COMBINED_COLOR: { return combined; }
        case BI_MEMORY_COLOR: { return vec3<f32>(0.0); } // TODO
        case BI_BLEND_COLOR: {
            return vec3<f32>(
                constants.blend_color[0],
                constants.blend_color[1],
                constants.blend_color[2],
            );
        }
        case BI_FOG_COLOR: {
            return vec3<f32>(
                constants.fog_color[0],
                constants.fog_color[1],
                constants.fog_color[2],
            );
        }
        default: { return vec3<f32>(0.0); }
    }
}

fn blend_factor_a(input: u32, shade_alpha: f32, combined: f32) -> f32 {
    switch input {
        case BFA_COMBINED_ALPHA: { return combined; }
        case BFA_FOG_ALPHA: { return constants.fog_color[3]; }
        case BFA_SHADE_ALPHA: { return shade_alpha; }
        case BFA_CONSTANT_0: { return 0.0; }
        default: { return 0.0; }
    }
}

fn blend_factor_b(input: u32, factor_a: f32) -> f32 {
    switch input {
        case BFB_ONE_MINUS_A: { return 1.0 - factor_a; }
        case BFB_MEMORY_ALPHA: { return 0.0; } // TODO
        case BFB_CONSTANT_1: { return 1.0; }
        case BFB_CONSTANT_0: { return 0.0; }
        default: { return 0.0; }
    }
}