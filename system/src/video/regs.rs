use bitfield_struct::bitfield;

#[bitfield(u32)]
pub struct Ctrl {
    #[bits(2)]
    color_mode: u32,
    gamma_dither_enable: bool,
    gamma_enable: bool,
    divot_enable: bool,
    vbus_clock_enable: bool,
    serrate: bool,
    test_mode: bool,
    #[bits(2)]
    aa_mode: u32,
    __: bool,
    kill_we: bool,
    #[bits(4)]
    pixel_advance: u32,
    dedither_enable: bool,
    #[bits(15)]
    __: u32,
}

#[bitfield(u32)]
pub struct Origin {
    #[bits(24)]
    pub origin: u32,
    #[bits(8)]
    __: u32,
}

#[bitfield(u32)]
pub struct Width {
    #[bits(12)]
    pub width: u32,
    #[bits(20)]
    __: u32,
}

#[bitfield(u32)]
pub struct VIntr {
    #[bits(10)]
    pub v_intr: u32,
    #[bits(22)]
    __: u32,
}

#[bitfield(u32)]
pub struct HVideo {
    #[bits(10)]
    pub h_end: u32,
    #[bits(6)]
    __: u32,
    #[bits(10)]
    pub h_start: u32,
    #[bits(6)]
    __: u32,
}
