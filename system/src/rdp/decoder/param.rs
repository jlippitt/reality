use super::{Context, Decoder};
use bitfield_struct::bitfield;
use tracing::trace;

pub fn set_blend_color(_decoder: &mut Decoder, ctx: Context, word: u64) {
    let cmd = SetBlendColor::from(word);
    trace!("{:?}", cmd);
    ctx.renderer.set_blend_color(cmd.color());
}

pub fn set_prim_depth(_decoder: &mut Decoder, ctx: Context, word: u64) {
    let cmd = SetPrimDepth::from(word);
    trace!("{:?}", cmd);
    ctx.renderer.set_prim_depth(cmd.z() as f32 / 65536.0);
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
