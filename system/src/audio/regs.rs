use bitfield_struct::bitfield;

#[bitfield(u32)]
pub struct DramAddr {
    #[bits(24)]
    pub dram_addr: u32,
    #[bits(8)]
    __: u32,
}

#[bitfield(u32)]
pub struct Length {
    #[bits(18)]
    pub length: u32,
    #[bits(14)]
    __: u32,
}
