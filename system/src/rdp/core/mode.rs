use super::{Bus, Core};
use bitfield_struct::bitfield;
use tracing::trace;

pub fn set_other_modes(_core: &mut Core, _bus: Bus, word: u64) {
    let cmd = SetOtherModes::from(word);
    trace!("{:?}", cmd);
    // TODO
}

#[bitfield(u64)]
struct SetOtherModes {
    alpha_compare_en: bool,
    dither_alpha_en: bool,
    z_source_sel: bool,
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
    // TODO: Split this up (once we implement blend)
    #[bits(16)]
    blend_modeword: u64,
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
    sample_type: bool,
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

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum CycleType {
    OneCycle = 0,
    TwoCycle = 1,
    Copy = 2,
    Fill = 3,
}

impl CycleType {
    const fn into_bits(self) -> u32 {
        self as u32
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
