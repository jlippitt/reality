use bitfield_struct::bitfield;

#[derive(Debug, Default)]
pub struct Regs {
    pub dma_sp_addr: DmaSpAddr,
    pub dma_ram_addr: DmaRamAddr,
    pub status: Status,
}

#[bitfield(u32)]
pub struct DmaSpAddr {
    #[bits(12)]
    mem_addr: u32,
    mem_bank: bool,
    #[bits(19)]
    __: u32,
}

#[bitfield(u32)]
pub struct DmaRamAddr {
    #[bits(12)]
    dram_addr: u32,
    #[bits(20)]
    __: u32,
}

#[bitfield(u32)]
pub struct Status {
    #[bits(default = true)]
    pub halted: bool,
    pub broke: bool,
    pub dma_busy: bool,
    pub dma_full: bool,
    pub io_busy: bool,
    pub sstep: bool,
    pub intbreak: bool,
    pub sig0: bool,
    pub sig1: bool,
    pub sig2: bool,
    pub sig3: bool,
    pub sig4: bool,
    pub sig5: bool,
    pub sig6: bool,
    pub sig7: bool,
    #[bits(17)]
    __: u32,
}
