use bitfield_struct::bitfield;

#[derive(Debug, Default)]
pub struct Regs {
    pub dram_addr: DramAddr,
    pub status: Status,
}

#[bitfield(u32)]
pub struct DramAddr {
    #[bits(24)]
    pub dram_addr: u32,
    #[bits(8)]
    __: u32,
}

#[bitfield(u32)]
pub struct Status {
    pub dma_busy: bool,
    pub io_busy: bool,
    pub read_pending: bool,
    pub dma_error: bool,
    #[bits(4)]
    pub pch_state: u32,
    #[bits(4)]
    pub dma_state: u32,
    pub interrupt: bool,
    #[bits(19)]
    __: u32,
}
