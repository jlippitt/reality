use bitfield_struct::bitfield;

#[bitfield(u32)]
pub struct Mode {
    #[bits(7)]
    pub repeat_count: u32,
    pub repeat: bool,
    pub ebus: bool,
    pub upper: bool,
    #[bits(22)]
    __: u32,
}

#[bitfield(u32)]
pub struct Mask {
    pub sp: bool,
    pub si: bool,
    pub ai: bool,
    pub vi: bool,
    pub pi: bool,
    pub dp: bool,
    #[bits(26)]
    __: u32,
}
