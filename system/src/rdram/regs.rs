use bitfield_struct::bitfield;

#[bitfield(u32)]
pub struct RiMode {
    #[bits(2)]
    pub op_mode: u32,
    pub stop_t: bool,
    pub stop_r: bool,
    #[bits(28)]
    __: u32,
}

#[bitfield(u32)]
pub struct RiConfig {
    #[bits(6)]
    pub cc: u32,
    pub auto_cc: bool,
    #[bits(25)]
    __: u32,
}

#[bitfield(u32)]
pub struct RiSelect {
    #[bits(4)]
    pub rsel: u32,
    #[bits(4)]
    pub tsel: u32,
    #[bits(24)]
    __: u32,
}
