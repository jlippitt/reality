use super::{Bus, Core};
use bitfield_struct::bitfield;
use tracing::trace;

pub fn set_fill_color(_core: &mut Core, bus: Bus, word: u64) {
    let cmd = SetFillColor::from(word);
    trace!("{:?}", cmd);
    bus.renderer.set_fill_color(cmd.packed_color());
}

pub fn set_blend_color(_core: &mut Core, bus: Bus, word: u64) {
    let cmd = SetBlendColor::from(word);
    trace!("{:?}", cmd);
    bus.renderer.set_blend_color(cmd.color());
}

pub fn set_prim_depth(_core: &mut Core, bus: Bus, word: u64) {
    let cmd = SetPrimDepth::from(word);
    trace!("{:?}", cmd);
    bus.renderer.set_prim_depth(cmd.z() as f32 / 65536.0);
}

#[bitfield(u64)]
struct SetFillColor {
    packed_color: u32,
    __: u32,
}

#[bitfield(u64)]
struct SetBlendColor {
    color: u32,
    __: u32,
}

#[bitfield(u64)]
struct SetPrimDepth {
    z: i32,
    __: u32,
}
