use bytemuck::{Pod, Zeroable};
use pod_enum::pod_enum;
use std::fmt::{self, Display, Formatter};
use tracing::trace;

#[allow(clippy::enum_variant_names)]
#[pod_enum]
#[repr(u8)]
#[derive(Eq)]
pub enum BlenderInput {
    CombinedColor = 0,
    MemoryColor = 1,
    BlendColor = 2,
    FogColor = 3,
}

#[pod_enum]
#[repr(u8)]
#[derive(Eq)]
pub enum BlendFactorA {
    CombinedAlpha = 0,
    FogAlpha = 1,
    ShadeAlpha = 2,
    Constant0 = 3,
}

#[pod_enum]
#[repr(u8)]
#[derive(Eq)]
pub enum BlendFactorB {
    OneMinusA = 0,
    MemoryAlpha = 1,
    Constant1 = 2,
    Constant0 = 3,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BlendModeRawParams {
    pub p: u32,
    pub a: u32,
    pub m: u32,
    pub b: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BlendModeRaw {
    pub mode: [BlendModeRawParams; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Pod, Zeroable)]
pub struct BlendModeParams {
    pub p: BlenderInput,
    pub a: BlendFactorA,
    pub m: BlenderInput,
    pub b: BlendFactorB,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Pod, Zeroable)]
pub struct BlendMode {
    pub mode: [BlendModeParams; 2],
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

impl BlendMode {
    pub fn from_raw(raw: BlendModeRaw) -> Self {
        let mode = Self {
            mode: raw.mode.map(BlendModeParams::from_raw),
        };

        trace!("  Blend Cycle 0: {}", mode.mode[0]);
        trace!("  Blend Cycle 1: {}", mode.mode[1]);

        mode
    }
}

impl Default for BlendMode {
    fn default() -> Self {
        Self::from_raw(BlendModeRaw::default())
    }
}
