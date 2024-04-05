use bitfield_struct::bitfield;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum RoundingMode {
    #[default]
    Round = 0,
    Trunc = 1,
    Ceil = 2,
    Floor = 3,
}

impl RoundingMode {
    const fn into_bits(self) -> u32 {
        self as u32
    }

    const fn from_bits(value: u32) -> Self {
        match value & 3 {
            0 => Self::Round,
            1 => Self::Trunc,
            2 => Self::Ceil,
            _ => Self::Floor,
        }
    }
}

#[bitfield(u32)]
pub struct Status {
    #[bits(2)]
    pub rm: RoundingMode,
    #[bits(5)]
    pub flags: u32,
    #[bits(5)]
    pub enables: u32,
    #[bits(6)]
    pub cause: u32,
    #[bits(5)]
    __: u32,
    pub c: bool,
    pub fs: bool,
    #[bits(7)]
    __: u32,
}
