use super::{Bus, Core};
use bitfield_struct::bitfield;
use tracing::trace;

pub fn sync_full(_core: &mut Core, _bus: Bus, word: u64) {
    let cmd = SyncFull::from(word);
    trace!("{:?}", cmd);
    // TODO
}

#[bitfield(u64)]
struct SyncFull {
    __: u64,
}
