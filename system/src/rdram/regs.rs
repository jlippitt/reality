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
pub struct Mode {
    #[bits(6)]
    __: u32,
    c0: bool,
    c3: bool,
    #[bits(6)]
    __: u32,
    c1: bool,
    c4: bool,
    #[bits(3)]
    __: u32,
    ad: bool,
    #[bits(2)]
    __: u32,
    c2: bool,
    c5: bool,
    le: bool,
    de: bool,
    as_: bool,
    sk: bool,
    sv: bool,
    pl: bool,
    x2: bool,
    ce: bool,
}

#[bitfield(u32)]
pub struct RefRow {
    #[bits(8)]
    __: u32,
    #[bits(2)]
    pub row_field_high: u32,
    #[bits(9)]
    __: u32,
    pub bank_field: bool,
    #[bits(5)]
    __: u32,
    #[bits(7)]
    pub row_field_low: u32,
}

#[bitfield(u32)]
pub struct RasInterval {
    #[bits(5)]
    pub row_exp_restore: u32,
    #[bits(3)]
    __: u32,
    #[bits(5)]
    pub row_imp_restore: u32,
    #[bits(3)]
    __: u32,
    #[bits(5)]
    pub row_sense: u32,
    #[bits(3)]
    __: u32,
    #[bits(5)]
    pub row_precharge: u32,
    #[bits(3)]
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
