use bitfield_struct::bitfield;
use std::mem;

#[allow(dead_code, clippy::upper_case_acronyms)]
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum Cp0Register {
    Index = 0,
    Random,
    EntryLo0,
    EntryLo1,
    Context,
    PageMask,
    Wired,
    R7,
    BadVAddr,
    Count,
    EntryHi,
    Compare,
    Status,
    Cause,
    EPC,
    PRId,
    Config,
    LLAddr,
    WatchLo,
    WatchHi,
    XContext,
    R21,
    R22,
    R23,
    R24,
    R25,
    ParityError,
    CacheError,
    TagLo,
    TagHi,
    ErrorEPC,
    R31,
}

impl From<u32> for Cp0Register {
    fn from(value: u32) -> Self {
        assert!(value <= 31);
        unsafe { mem::transmute::<u32, Self>(value) }
    }
}

#[bitfield(u32)]
pub struct Status {
    pub ie: bool,
    pub exl: bool,
    pub erl: bool,
    #[bits(2)]
    pub ksu: u32,
    pub ux: bool,
    pub sx: bool,
    pub kx: bool,
    pub im: u8,
    #[bits(9)]
    pub ds: u32,
    pub re: bool,
    pub fr: bool,
    pub rp: bool,
    pub cu0: bool,
    pub cu1: bool,
    pub cu2: bool,
    pub cu3: bool,
}
