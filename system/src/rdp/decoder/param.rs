use super::renderer::FixedColor;
use super::{Context, Decoder};
use bitfield_struct::bitfield;
use tracing::trace;

pub fn set_fog_color(_decoder: &mut Decoder, ctx: Context, word: u64) {
    let cmd = SetFogColor::from(word);
    trace!("{:?}", cmd);
    ctx.renderer.set_fixed_color(FixedColor::Fog, cmd.color());
}

pub fn set_blend_color(_decoder: &mut Decoder, ctx: Context, word: u64) {
    let cmd = SetBlendColor::from(word);
    trace!("{:?}", cmd);
    ctx.renderer.set_fixed_color(FixedColor::Blend, cmd.color());
}

pub fn set_prim_color(_decoder: &mut Decoder, ctx: Context, word: u64) {
    let cmd = SetPrimColor::from(word);
    trace!("{:?}", cmd);
    // TODO: LOD params
    ctx.renderer
        .set_fixed_color(FixedColor::Primitive, cmd.color());
}

pub fn set_env_color(_decoder: &mut Decoder, ctx: Context, word: u64) {
    let cmd = SetEnvColor::from(word);
    trace!("{:?}", cmd);
    ctx.renderer
        .set_fixed_color(FixedColor::Environment, cmd.color());
}

pub fn set_prim_depth(_decoder: &mut Decoder, ctx: Context, word: u64) {
    let cmd = SetPrimDepth::from(word);
    trace!("{:?}", cmd);
    ctx.renderer.set_prim_depth(cmd.z() as f32);
}

#[bitfield(u64)]
struct SetFogColor {
    color: u32,
    __: u32,
}

#[bitfield(u64)]
struct SetBlendColor {
    color: u32,
    __: u32,
}

#[bitfield(u64)]
struct SetPrimColor {
    color: u32,
    #[bits(8)]
    prim_lod_frac: u32,
    #[bits(5)]
    prim_min_level: u32,
    #[bits(19)]
    __: u32,
}

#[bitfield(u64)]
struct SetEnvColor {
    color: u32,
    __: u32,
}

#[bitfield(u64)]
struct SetPrimDepth {
    delta_z: i16,
    z: i16,
    __: u32,
}
