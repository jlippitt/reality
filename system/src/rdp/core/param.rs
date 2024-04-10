use super::{Bus, Core};
use bitfield_struct::bitfield;
use tracing::trace;

pub fn set_fill_color(_core: &mut Core, _bus: Bus, word: u64) {
    let cmd = SetFillColor::from(word);
    trace!("{:?}", cmd);
    // TODO
}

#[bitfield(u64)]
struct SetFillColor {
    packed_color: u32,
    __: u32,
}