use bytemuck::{Pod, Zeroable};
use pod_enum::pod_enum;
use std::fmt::{self, Display, Formatter};

#[pod_enum]
#[repr(u32)]
#[derive(Eq)]
pub enum CombinerInput {
    CombinedColor = 0,
    Texel0Color = 1,
    Texel1Color = 2,
    PrimColor = 3,
    ShadeColor = 4,
    EnvColor = 5,
    KeyCenter = 6,
    KeyScale = 7,
    CombinedAlpha = 8,
    Texel0Alpha = 9,
    Texel1Alpha = 10,
    PrimAlpha = 11,
    ShadeAlpha = 12,
    EnvAlpha = 13,
    LodFraction = 14,
    PrimLodFraction = 15,
    Noise = 16,
    ConvertK4 = 17,
    ConvertK5 = 18,
    Constant1 = 19,
    Constant0 = 20,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CombineModeRawParams {
    pub sub_a: u32,
    pub sub_b: u32,
    pub mul: u32,
    pub add: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CombineModeRaw {
    pub rgb: [CombineModeRawParams; 2],
    pub alpha: [CombineModeRawParams; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Pod, Zeroable)]
pub struct CombineModeParams {
    pub sub_a: CombinerInput,
    pub sub_b: CombinerInput,
    pub mul: CombinerInput,
    pub add: CombinerInput,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Pod, Zeroable)]
pub struct CombineMode {
    pub rgb: [CombineModeParams; 2],
    pub alpha: [CombineModeParams; 2],
}

impl CombineMode {
    pub fn from_raw(raw: CombineModeRaw) -> Self {
        Self {
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

impl Default for CombineMode {
    fn default() -> Self {
        Self::from_raw(CombineModeRaw::default())
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
