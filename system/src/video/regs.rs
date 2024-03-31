use bitfield_struct::bitfield;

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
