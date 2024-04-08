use bitfield_struct::bitfield;

#[derive(Debug, Default)]
pub struct Regs {
    pub dram_addr: DramAddr,
    pub control: Control,
    pub dacrate: Dacrate,
    pub bitrate: Bitrate,
}

#[bitfield(u32)]
pub struct DramAddr {
    #[bits(24)]
    pub dram_addr: u32,
    #[bits(8)]
    __: u32,
}

#[bitfield(u32)]
pub struct Control {
    pub dma_enable: bool,
    #[bits(31)]
    __: u32,
}

#[bitfield(u32)]
pub struct Dacrate {
    #[bits(14)]
    pub dacrate: u32,
    #[bits(18)]
    __: u32,
}

#[bitfield(u32)]
pub struct Bitrate {
    #[bits(4)]
    pub bitrate: u32,
    #[bits(28)]
    __: u32,
}
