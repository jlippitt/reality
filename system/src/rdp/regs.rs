use bitfield_struct::bitfield;

#[bitfield(u32)]
pub struct Status {
    xbus: bool,
    freeze: bool,
    flush: bool,
    start_gclk: bool,
    tmem_busy: bool,
    pipe_busy: bool,
    buf_busy: bool,
    cbuf_ready: bool,
    dma_busy: bool,
    end_pending: bool,
    start_pending: bool,
    #[bits(21)]
    __: u32,
}
