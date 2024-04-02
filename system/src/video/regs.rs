use bitfield_struct::bitfield;

#[derive(Default)]
pub struct Regs {
    pub ctrl: Ctrl,
    pub origin: Origin,
    pub width: Width,
    pub v_intr: VIntr,
    pub burst: Burst,
    pub v_sync: VSync,
    pub h_sync: HSync,
    pub h_sync_leap: HSyncLeap,
    pub h_video: Range,
    pub v_video: Range,
    pub v_burst: Range,
    pub x_scale: Scale,
    pub y_scale: Scale,
    pub test_addr: TestAddr,
    pub staged_data: u32,
}

#[bitfield(u32)]
pub struct Ctrl {
    #[bits(2)]
    pub color_mode: u32,
    pub gamma_dither_enable: bool,
    pub gamma_enable: bool,
    pub divot_enable: bool,
    pub vbus_clock_enable: bool,
    pub serrate: bool,
    pub test_mode: bool,
    #[bits(2)]
    pub aa_mode: u32,
    __: bool,
    pub kill_we: bool,
    #[bits(4)]
    pub pixel_advance: u32,
    pub dedither_enable: bool,
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
pub struct Burst {
    #[bits(8)]
    pub hsync_width: u32,
    #[bits(8)]
    pub burst_width: u32,
    #[bits(4)]
    pub vsync_width: u32,
    #[bits(10)]
    pub burst_start: u32,
    #[bits(2)]
    __: u32,
}

#[bitfield(u32)]
pub struct VSync {
    #[bits(10)]
    pub v_sync: u32,
    #[bits(22)]
    __: u32,
}

#[bitfield(u32)]
pub struct HSync {
    #[bits(12)]
    pub h_sync: u32,
    #[bits(4)]
    __: u32,
    #[bits(5)]
    pub leap: u32,
    #[bits(11)]
    __: u32,
}

#[bitfield(u32)]
pub struct HSyncLeap {
    #[bits(12)]
    pub leap_b: u32,
    #[bits(4)]
    __: u32,
    #[bits(12)]
    pub leap_a: u32,
    #[bits(4)]
    __: u32,
}

#[bitfield(u32)]
pub struct Range {
    #[bits(10)]
    pub end: u32,
    #[bits(6)]
    __: u32,
    #[bits(10)]
    pub start: u32,
    #[bits(6)]
    __: u32,
}

#[bitfield(u32)]
pub struct Scale {
    #[bits(12)]
    pub x_scale: u32,
    #[bits(4)]
    __: u32,
    #[bits(12)]
    pub x_offset: u32,
    #[bits(4)]
    __: u32,
}

#[bitfield(u32)]
pub struct TestAddr {
    #[bits(7)]
    pub test_addr: u32,
    #[bits(25)]
    __: u32,
}
