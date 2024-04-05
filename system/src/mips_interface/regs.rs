use bitfield_struct::bitfield;

#[derive(Debug, Default)]
pub struct Regs {
    pub mode: Mode,
}

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
