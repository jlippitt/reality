use bitfield_struct::bitfield;

#[bitfield(u32)]
pub struct Delay {
    #[bits(3, access = RO)]
    pub write_bits: u32,
    #[bits(3, default = 4)]
    pub write_delay: u32,
    #[bits(2)]
    __: u32,
    #[bits(3, access = RO)]
    pub ack_bits: u32,
    #[bits(2)]
    pub ack_delay: u32,
    #[bits(3)]
    __: u32,
    #[bits(3, access = RO)]
    pub read_bits: u32,
    #[bits(3, default = 1)]
    pub read_delay: u32,
    #[bits(2)]
    __: u32,
    #[bits(3, access = RO)]
    pub ack_win_bits: u32,
    #[bits(3)]
    pub ack_win_delay: u32,
    #[bits(2)]
    __: u32,
}

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
