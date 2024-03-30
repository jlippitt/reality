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
    pub sig: u8,
    #[bits(17)]
    __: u32,
}
