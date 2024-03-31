use bitfield_struct::bitfield;

#[bitfield(u32)]
pub struct Status {
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
