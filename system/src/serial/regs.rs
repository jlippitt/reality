use bitfield_struct::bitfield;

#[bitfield(u32)]
pub struct DramAddr {
    #[bits(24)]
    pub dram_addr: u32,
    #[bits(8)]
    __: u32,
}

#[bitfield(u32)]
pub struct Status {
    dma_busy: bool,
    io_busy: bool,
    read_pending: bool,
    dma_error: bool,
    #[bits(4)]
    pch_state: u32,
    #[bits(4)]
    dma_state: u32,
    interrupt: bool,
    #[bits(19)]
    __: u32,
}
