use std::array;
use std::fmt::{self, Display, Formatter};
use tracing::trace;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum CombinerInput {
    CombinedColor = 0,
    Texel0Color,
    Texel1Color,
    PrimColor,
    ShadeColor,
    EnvColor,
    KeyCenter,
    KeyScale,
    CombinedAlpha,
    Texel0Alpha,
    Texel1Alpha,
    PrimAlpha,
    ShadeAlpha,
    EnvAlpha,
    LodFraction,
    PrimLodFraction,
    Noise,
    ConvertK4,
    ConvertK5,
    Constant1,
    Constant0,
}

#[allow(clippy::enum_variant_names)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum BlenderInput {
    CombinedColor = 0,
    MemoryColor,
    BlendColor,
    FogColor,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum BlendFactorA {
    CombinedAlpha = 0,
    FogAlpha,
    ShadeAlpha,
    Constant0,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum BlendFactorB {
    OneMinusA = 0,
    MemoryAlpha,
    Constant1,
    Constant0,
}

#[derive(Debug, Default)]
pub struct CombineModeRawParams {
    pub sub_a: u32,
    pub sub_b: u32,
    pub mul: u32,
    pub add: u32,
}

#[derive(Debug, Default)]
pub struct CombineModeRaw {
    pub rgb: [CombineModeRawParams; 2],
    pub alpha: [CombineModeRawParams; 2],
}

#[derive(Debug)]
struct CombineModeParams {
    pub sub_a: CombinerInput,
    pub sub_b: CombinerInput,
    pub mul: CombinerInput,
    pub add: CombinerInput,
}

#[derive(Debug)]
struct CombineMode {
    pub rgb: [CombineModeParams; 2],
    pub alpha: [CombineModeParams; 2],
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BlendModeRawParams {
    pub p: u32,
    pub a: u32,
    pub m: u32,
    pub b: u32,
}

pub type BlendModeRaw = [BlendModeRawParams; 2];

#[derive(Debug)]
struct BlendModeParams {
    pub p: BlenderInput,
    pub a: BlendFactorA,
    pub m: BlenderInput,
    pub b: BlendFactorB,
}

type BlendMode = [BlendModeParams; 2];

pub struct Combiner {
    combine_mode: CombineMode,
    blend_mode: BlendMode,
    hash_value: u64,
}

impl Combiner {
    pub fn new() -> Self {
        Self {
            combine_mode: CombineMode::from_raw(CombineModeRaw::default()),
            blend_mode: array::from_fn(
                |_| BlendModeParams::from_raw(BlendModeRawParams::default()),
            ),
            hash_value: 0,
        }
    }

    pub fn hash_value(&self) -> u64 {
        self.hash_value
    }

    pub fn set_combine_mode(&mut self, combine_mode: CombineModeRaw, hash_value: u64) {
        self.combine_mode = CombineMode::from_raw(combine_mode);
        self.hash_value = hash_value;
        trace!("  RGB Cycle 0: {}", self.combine_mode.rgb[0]);
        trace!("  RGB Cycle 1: {}", self.combine_mode.rgb[1]);
        trace!("  Alpha Cycle 0: {}", self.combine_mode.alpha[0]);
        trace!("  Alpha Cycle 1: {}", self.combine_mode.alpha[1]);
        trace!("  Combine Mode Hash Value: {:08X}", self.hash_value);
    }

    pub fn set_blend_mode(&mut self, blend_mode: BlendModeRaw) {
        self.blend_mode = blend_mode.map(BlendModeParams::from_raw);
        trace!("  Blend Cycle 0: {}", self.blend_mode[0]);
        trace!("  Blend Cycle 1: {}", self.blend_mode[1]);
    }
}

impl CombineMode {
    fn from_raw(raw: CombineModeRaw) -> Self {
        CombineMode {
            rgb: raw.rgb.map(
                |CombineModeRawParams {
                     sub_a,
                     sub_b,
                     mul,
                     add,
                 }| CombineModeParams {
                    sub_a: match sub_a {
                        0 => CombinerInput::CombinedColor,
                        1 => CombinerInput::Texel0Color,
                        2 => CombinerInput::Texel1Color,
                        3 => CombinerInput::PrimColor,
                        4 => CombinerInput::ShadeColor,
                        5 => CombinerInput::EnvColor,
                        6 => CombinerInput::Constant1,
                        7 => CombinerInput::Noise,
                        _ => CombinerInput::Constant0,
                    },
                    sub_b: match sub_b {
                        0 => CombinerInput::CombinedColor,
                        1 => CombinerInput::Texel0Color,
                        2 => CombinerInput::Texel1Color,
                        3 => CombinerInput::PrimColor,
                        4 => CombinerInput::ShadeColor,
                        5 => CombinerInput::EnvColor,
                        6 => CombinerInput::KeyCenter,
                        7 => CombinerInput::ConvertK4,
                        _ => CombinerInput::Constant0,
                    },
                    mul: match mul {
                        0 => CombinerInput::CombinedColor,
                        1 => CombinerInput::Texel0Color,
                        2 => CombinerInput::Texel1Color,
                        3 => CombinerInput::PrimColor,
                        4 => CombinerInput::ShadeColor,
                        5 => CombinerInput::EnvColor,
                        6 => CombinerInput::KeyScale,
                        7 => CombinerInput::CombinedAlpha,
                        8 => CombinerInput::Texel0Alpha,
                        9 => CombinerInput::Texel1Alpha,
                        10 => CombinerInput::PrimAlpha,
                        11 => CombinerInput::ShadeAlpha,
                        12 => CombinerInput::EnvAlpha,
                        13 => CombinerInput::LodFraction,
                        14 => CombinerInput::PrimLodFraction,
                        15 => CombinerInput::ConvertK5,
                        _ => CombinerInput::Constant0,
                    },
                    add: match add {
                        0 => CombinerInput::CombinedColor,
                        1 => CombinerInput::Texel0Color,
                        2 => CombinerInput::Texel1Color,
                        3 => CombinerInput::PrimColor,
                        4 => CombinerInput::ShadeColor,
                        5 => CombinerInput::EnvColor,
                        6 => CombinerInput::Constant1,
                        _ => CombinerInput::Constant0,
                    },
                },
            ),
            alpha: raw.alpha.map(
                |CombineModeRawParams {
                     sub_a,
                     sub_b,
                     mul,
                     add,
                 }| CombineModeParams {
                    sub_a: match sub_a {
                        0 => CombinerInput::CombinedAlpha,
                        1 => CombinerInput::Texel0Alpha,
                        2 => CombinerInput::Texel1Alpha,
                        3 => CombinerInput::PrimAlpha,
                        4 => CombinerInput::ShadeAlpha,
                        5 => CombinerInput::EnvAlpha,
                        6 => CombinerInput::Constant1,
                        _ => CombinerInput::Constant0,
                    },
                    sub_b: match sub_b {
                        0 => CombinerInput::CombinedAlpha,
                        1 => CombinerInput::Texel0Alpha,
                        2 => CombinerInput::Texel1Alpha,
                        3 => CombinerInput::PrimAlpha,
                        4 => CombinerInput::ShadeAlpha,
                        5 => CombinerInput::EnvAlpha,
                        6 => CombinerInput::Constant1,
                        _ => CombinerInput::Constant0,
                    },
                    mul: match mul {
                        0 => CombinerInput::LodFraction,
                        1 => CombinerInput::Texel0Alpha,
                        2 => CombinerInput::Texel1Alpha,
                        3 => CombinerInput::PrimAlpha,
                        4 => CombinerInput::ShadeAlpha,
                        5 => CombinerInput::EnvAlpha,
                        6 => CombinerInput::PrimLodFraction,
                        _ => CombinerInput::Constant0,
                    },
                    add: match add {
                        0 => CombinerInput::CombinedAlpha,
                        1 => CombinerInput::Texel0Alpha,
                        2 => CombinerInput::Texel1Alpha,
                        3 => CombinerInput::PrimAlpha,
                        4 => CombinerInput::ShadeAlpha,
                        5 => CombinerInput::EnvAlpha,
                        6 => CombinerInput::Constant1,
                        _ => CombinerInput::Constant0,
                    },
                },
            ),
        }
    }
}

impl Display for CombineModeParams {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({:?} - {:?}) * {:?} + {:?}",
            self.sub_a, self.sub_b, self.mul, self.add,
        )
    }
}

impl BlenderInput {
    fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::CombinedColor,
            1 => Self::MemoryColor,
            2 => Self::BlendColor,
            _ => Self::FogColor,
        }
    }
}

impl BlendFactorA {
    fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::CombinedAlpha,
            1 => Self::FogAlpha,
            2 => Self::ShadeAlpha,
            _ => Self::Constant0,
        }
    }
}

impl BlendFactorB {
    fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::OneMinusA,
            1 => Self::MemoryAlpha,
            2 => Self::Constant1,
            _ => Self::Constant0,
        }
    }
}

impl BlendModeParams {
    fn from_raw(raw: BlendModeRawParams) -> Self {
        Self {
            p: BlenderInput::from_raw(raw.p),
            m: BlenderInput::from_raw(raw.m),
            a: BlendFactorA::from_raw(raw.a),
            b: BlendFactorB::from_raw(raw.b),
        }
    }
}

impl Display for BlendModeParams {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} * {:?} + {:?} * {:?}",
            self.p, self.a, self.m, self.b
        )
    }
}
