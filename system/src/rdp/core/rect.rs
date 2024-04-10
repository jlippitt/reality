use super::{Bus, Core};
use bitfield_struct::bitfield;
use tracing::trace;

pub fn fill_rectangle(_core: &mut Core, _bus: Bus, word: u64) {
    let cmd = FillRectangle::from(word);
    trace!("{:?}", cmd);
    // TODO
}

#[bitfield(u64)]
struct FillRectangle {
    #[bits(12)]
    yh: u64,
    #[bits(12)]
    xh: u64,
    #[bits(8)]
    __: u64,
    #[bits(12)]
    yl: u64,
    #[bits(12)]
    xl: u64,
    #[bits(8)]
    __: u64,
}
