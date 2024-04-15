use bitfield_struct::bitfield;

#[derive(Debug, Default)]
pub struct Regs {
    pub start: u32,
    pub end: u32,
    pub status: Status,
}

#[bitfield(u32)]
pub struct Status {
    pub xbus: bool,
    pub freeze: bool,
    pub flush: bool,
    pub start_gclk: bool,
    pub tmem_busy: bool,
    pub pipe_busy: bool,
    pub buf_busy: bool,
    #[bits(default = true)]
    pub cbuf_ready: bool,
    pub dma_busy: bool,
    pub end_pending: bool,
    pub start_pending: bool,
    #[bits(21)]
    __: u32,
}
