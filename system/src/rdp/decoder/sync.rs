use super::{Context, Decoder};
use bitfield_struct::bitfield;
use tracing::trace;

pub fn sync_load(_decoder: &mut Decoder, _ctx: Context, word: u64) {
    let cmd = SyncLoad::from(word);
    trace!("{:?}", cmd);
    // TODO
}

pub fn sync_pipe(_decoder: &mut Decoder, _ctx: Context, word: u64) {
    let cmd = SyncPipe::from(word);
    trace!("{:?}", cmd);
    // TODO
}

pub fn sync_tile(_decoder: &mut Decoder, _ctx: Context, word: u64) {
    let cmd = SyncTile::from(word);
    trace!("{:?}", cmd);
    // TODO
}

pub fn sync_full(_decoder: &mut Decoder, _ctx: Context, word: u64) {
    let cmd = SyncFull::from(word);
    trace!("{:?}", cmd);
    // TODO
}

#[bitfield(u64)]
struct SyncLoad {
    __: u64,
}

#[bitfield(u64)]
struct SyncPipe {
    __: u64,
}

#[bitfield(u64)]
struct SyncTile {
    __: u64,
}

#[bitfield(u64)]
struct SyncFull {
    __: u64,
}
