use super::renderer::{
    BlendModeRaw, BlendModeRawParams, CombineModeRaw, CombineModeRawParams, CycleType, OtherModes,
    SampleType, ZSource,
};
use super::{Context, Decoder};
use bitfield_struct::bitfield;
use std::mem;
use tracing::trace;

pub fn set_combine_mode(_decoder: &mut Decoder, ctx: Context, word: u64) {
    let cmd = SetCombineMode::from(word);

    trace!("{:?}", cmd);

    ctx.renderer.set_combine_mode(CombineModeRaw {
        rgb: [
            CombineModeRawParams {
                sub_a: cmd.sub_a_r_0(),
                sub_b: cmd.sub_b_r_0(),
                mul: cmd.mul_r_0(),
                add: cmd.add_r_0(),
            },
            CombineModeRawParams {
                sub_a: cmd.sub_a_r_1(),
                sub_b: cmd.sub_b_r_1(),
                mul: cmd.mul_r_1(),
                add: cmd.add_r_1(),
            },
        ],
        alpha: [
            CombineModeRawParams {
                sub_a: cmd.sub_a_a_0(),
                sub_b: cmd.sub_b_a_0(),
                mul: cmd.mul_a_0(),
                add: cmd.add_a_0(),
            },
            CombineModeRawParams {
                sub_a: cmd.sub_a_a_1(),
                sub_b: cmd.sub_b_a_1(),
                mul: cmd.mul_a_1(),
                add: cmd.add_a_1(),
            },
        ],
    });
}

pub fn set_other_modes(_decoder: &mut Decoder, ctx: Context, word: u64) {
    let cmd = SetOtherModes::from(word);

    trace!("{:?}", cmd);

    ctx.renderer.set_other_modes(
        ctx.gfx,
        ctx.rdram,
        OtherModes {
            cycle_type: cmd.cycle_type(),
            sample_type: cmd.sample_type(),
            perspective_enable: cmd.persp_tex_en(),
            z_compare_en: cmd.z_compare_en(),
            z_update_en: cmd.z_update_en(),
            z_source: cmd.z_source_sel(),
            blend_mode: BlendModeRaw {
                mode: [
                    BlendModeRawParams {
                        p: cmd.b_m1a_0(),
                        a: cmd.b_m1b_0(),
                        m: cmd.b_m2a_0(),
                        b: cmd.b_m2b_0(),
                    },
                    BlendModeRawParams {
                        p: cmd.b_m1a_1(),
                        a: cmd.b_m1b_1(),
                        m: cmd.b_m2a_1(),
                        b: cmd.b_m2b_1(),
                    },
                ],
            },
        },
    );
}

#[bitfield(u64)]
struct SetCombineMode {
    #[bits(3)]
    add_a_1: u32,
    #[bits(3)]
    sub_b_a_1: u32,
    #[bits(3)]
    add_r_1: u32,
    #[bits(3)]
    add_a_0: u32,
    #[bits(3)]
    sub_b_a_0: u32,
    #[bits(3)]
    add_r_0: u32,
    #[bits(3)]
    mul_a_1: u32,
    #[bits(3)]
    sub_a_a_1: u32,
    #[bits(4)]
    sub_b_r_1: u32,
    #[bits(4)]
    sub_b_r_0: u32,
    #[bits(5)]
    mul_r_1: u32,
    #[bits(4)]
    sub_a_r_1: u32,
    #[bits(3)]
    mul_a_0: u32,
    #[bits(3)]
    sub_a_a_0: u32,
    #[bits(5)]
    mul_r_0: u32,
    #[bits(4)]
    sub_a_r_0: u32,
    #[bits(8)]
    __: u64,
}

#[bitfield(u64)]
struct SetOtherModes {
    alpha_compare_en: bool,
    dither_alpha_en: bool,
    #[bits(1)]
    z_source_sel: ZSource,
    antialias_en: bool,
    z_compare_en: bool,
    z_update_en: bool,
    image_read_en: bool,
    color_on_cvg: bool,
    #[bits(2)]
    cvg_dest: CvgDest,
    #[bits(2)]
    z_mode: ZMode,
    cvg_times_alpha: bool,
    alpha_cvg_select: bool,
    force_blend: bool,
    __: bool,
    #[bits(2)]
    b_m2b_1: u32,
    #[bits(2)]
    b_m2b_0: u32,
    #[bits(2)]
    b_m2a_1: u32,
    #[bits(2)]
    b_m2a_0: u32,
    #[bits(2)]
    b_m1b_1: u32,
    #[bits(2)]
    b_m1b_0: u32,
    #[bits(2)]
    b_m1a_1: u32,
    #[bits(2)]
    b_m1a_0: u32,
    #[bits(4)]
    __: u64,
    #[bits(2)]
    alpha_dither_sel: AlphaDitherSelect,
    #[bits(2)]
    rgb_dither_sel: RgbDitherSelect,
    key_en: bool,
    convert_one: bool,
    bi_lerp_0: bool,
    bi_lerp_1: bool,
    mid_texel: bool,
    #[bits(1)]
    sample_type: SampleType,
    tlut_type: bool,
    en_tlut: bool,
    tex_lod_en: bool,
    sharpen_tex_en: bool,
    detail_tex_en: bool,
    persp_tex_en: bool,
    #[bits(2)]
    cycle_type: CycleType,
    __: bool,
    atomic_prim: bool,
    #[bits(8)]
    __: u64,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum CvgDest {
    Clamp = 0,
    Wrap = 1,
    Zap = 2,
    Save = 3,
}

impl CvgDest {
    const fn into_bits(self) -> u32 {
        self as u32
    }

    const fn from_bits(value: u32) -> Self {
        match value & 3 {
            0 => Self::Clamp,
            1 => Self::Wrap,
            2 => Self::Zap,
            _ => Self::Save,
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ZMode {
    Opaque = 0,
    Interpenetrating = 1,
    Transparent = 2,
    Decal = 3,
}

impl ZMode {
    const fn into_bits(self) -> u32 {
        self as u32
    }

    const fn from_bits(value: u32) -> Self {
        match value & 3 {
            0 => Self::Opaque,
            1 => Self::Interpenetrating,
            2 => Self::Transparent,
            _ => Self::Decal,
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum AlphaDitherSelect {
    Pattern = 0,
    PatternInverted = 1,
    Noise = 2,
    NoDither = 3,
}

impl AlphaDitherSelect {
    const fn into_bits(self) -> u32 {
        self as u32
    }

    const fn from_bits(value: u32) -> Self {
        match value & 3 {
            0 => Self::Pattern,
            1 => Self::PatternInverted,
            2 => Self::Noise,
            _ => Self::NoDither,
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum RgbDitherSelect {
    MagicSquare = 0,
    Bayer = 1,
    Noise = 2,
    NoDither = 3,
}

impl RgbDitherSelect {
    const fn into_bits(self) -> u32 {
        self as u32
    }

    const fn from_bits(value: u32) -> Self {
        match value & 3 {
            0 => Self::MagicSquare,
            1 => Self::Bayer,
            2 => Self::Noise,
            _ => Self::NoDither,
        }
    }
}

impl CycleType {
    const fn into_bits(self) -> u32 {
        unsafe { mem::transmute(self) }
    }

    const fn from_bits(value: u32) -> Self {
        match value & 3 {
            0 => Self::OneCycle,
            1 => Self::TwoCycle,
            2 => Self::Copy,
            _ => Self::Fill,
        }
    }
}

impl SampleType {
    const fn into_bits(self) -> u32 {
        self as u32
    }

    const fn from_bits(value: u32) -> Self {
        match value & 1 {
            0 => Self::Point,
            _ => Self::Bilinear,
        }
    }
}

impl ZSource {
    const fn into_bits(self) -> u32 {
        self as u32
    }

    const fn from_bits(value: u32) -> Self {
        match value & 1 {
            0 => Self::PerPixel,
            _ => Self::Primitive,
        }
    }
}
