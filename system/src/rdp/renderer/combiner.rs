use std::fmt::{self, Debug, Display, Formatter};
use tracing::trace;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Input {
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

#[derive(Debug)]
pub struct CombineModeParams<T> {
    pub sub_a: T,
    pub sub_b: T,
    pub mul: T,
    pub add: T,
}

#[derive(Debug)]
pub struct CombineMode<T> {
    pub rgb: [CombineModeParams<T>; 2],
    pub alpha: [CombineModeParams<T>; 2],
}

pub struct Combiner {
    combine_mode: CombineMode<Input>,
    hash_value: u64,
}

impl Combiner {
    pub fn new() -> Self {
        Self {
            combine_mode: CombineMode::from_raw(CombineMode::default()),
            hash_value: 0,
        }
    }

    pub fn hash_value(&self) -> u64 {
        self.hash_value
    }

    pub fn set_combine_mode(&mut self, combine_mode: CombineMode<u32>, hash_value: u64) {
        self.combine_mode = CombineMode::from_raw(combine_mode);
        self.hash_value = hash_value;
        trace!("  RGB Cycle 0: {}", self.combine_mode.rgb[0]);
        trace!("  RGB Cycle 1: {}", self.combine_mode.rgb[1]);
        trace!("  Alpha Cycle 0: {}", self.combine_mode.alpha[0]);
        trace!("  Alpha Cycle 1: {}", self.combine_mode.alpha[1]);
        trace!("  Combine Mode Hash Value: {:08X}", self.hash_value);
    }
}

impl<T> Default for CombineModeParams<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            sub_a: T::default(),
            sub_b: T::default(),
            mul: T::default(),
            add: T::default(),
        }
    }
}

impl<T> Default for CombineMode<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            rgb: [CombineModeParams::default(), CombineModeParams::default()],
            alpha: [CombineModeParams::default(), CombineModeParams::default()],
        }
    }
}

impl CombineMode<Input> {
    fn from_raw(raw: CombineMode<u32>) -> Self {
        CombineMode {
            rgb: raw.rgb.map(
                |CombineModeParams {
                     sub_a,
                     sub_b,
                     mul,
                     add,
                 }| CombineModeParams {
                    sub_a: match sub_a {
                        0 => Input::CombinedColor,
                        1 => Input::Texel0Color,
                        2 => Input::Texel1Color,
                        3 => Input::PrimColor,
                        4 => Input::ShadeColor,
                        5 => Input::EnvColor,
                        6 => Input::Constant1,
                        7 => Input::Noise,
                        _ => Input::Constant0,
                    },
                    sub_b: match sub_b {
                        0 => Input::CombinedColor,
                        1 => Input::Texel0Color,
                        2 => Input::Texel1Color,
                        3 => Input::PrimColor,
                        4 => Input::ShadeColor,
                        5 => Input::EnvColor,
                        6 => Input::KeyCenter,
                        7 => Input::ConvertK4,
                        _ => Input::Constant0,
                    },
                    mul: match mul {
                        0 => Input::CombinedColor,
                        1 => Input::Texel0Color,
                        2 => Input::Texel1Color,
                        3 => Input::PrimColor,
                        4 => Input::ShadeColor,
                        5 => Input::EnvColor,
                        6 => Input::KeyScale,
                        7 => Input::CombinedAlpha,
                        8 => Input::Texel0Alpha,
                        9 => Input::Texel1Alpha,
                        10 => Input::PrimAlpha,
                        11 => Input::ShadeAlpha,
                        12 => Input::EnvAlpha,
                        13 => Input::LodFraction,
                        14 => Input::PrimLodFraction,
                        15 => Input::ConvertK5,
                        _ => Input::Constant0,
                    },
                    add: match add {
                        0 => Input::CombinedColor,
                        1 => Input::Texel0Color,
                        2 => Input::Texel1Color,
                        3 => Input::PrimColor,
                        4 => Input::ShadeColor,
                        5 => Input::EnvColor,
                        6 => Input::Constant1,
                        _ => Input::Constant0,
                    },
                },
            ),
            alpha: raw.alpha.map(
                |CombineModeParams {
                     sub_a,
                     sub_b,
                     mul,
                     add,
                 }| CombineModeParams {
                    sub_a: match sub_a {
                        0 => Input::CombinedAlpha,
                        1 => Input::Texel0Alpha,
                        2 => Input::Texel1Alpha,
                        3 => Input::PrimAlpha,
                        4 => Input::ShadeAlpha,
                        5 => Input::EnvAlpha,
                        6 => Input::Constant1,
                        _ => Input::Constant0,
                    },
                    sub_b: match sub_b {
                        0 => Input::CombinedAlpha,
                        1 => Input::Texel0Alpha,
                        2 => Input::Texel1Alpha,
                        3 => Input::PrimAlpha,
                        4 => Input::ShadeAlpha,
                        5 => Input::EnvAlpha,
                        6 => Input::Constant1,
                        _ => Input::Constant0,
                    },
                    mul: match mul {
                        0 => Input::LodFraction,
                        1 => Input::Texel0Alpha,
                        2 => Input::Texel1Alpha,
                        3 => Input::PrimAlpha,
                        4 => Input::ShadeAlpha,
                        5 => Input::EnvAlpha,
                        6 => Input::PrimLodFraction,
                        _ => Input::Constant0,
                    },
                    add: match add {
                        0 => Input::CombinedAlpha,
                        1 => Input::Texel0Alpha,
                        2 => Input::Texel1Alpha,
                        3 => Input::PrimAlpha,
                        4 => Input::ShadeAlpha,
                        5 => Input::EnvAlpha,
                        6 => Input::Constant1,
                        _ => Input::Constant0,
                    },
                },
            ),
        }
    }
}

impl Display for Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl<T> Display for CombineModeParams<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({} - {}) * {} + {}",
            self.sub_a, self.sub_b, self.mul, self.add,
        )
    }
}
