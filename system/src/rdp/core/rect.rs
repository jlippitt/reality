use super::renderer::Rect;
use super::{Bus, Core};
use bitfield_struct::bitfield;
use tracing::trace;

pub fn fill_rectangle(_core: &mut Core, bus: Bus, word: u64) {
    let cmd = FillRectangle::from(word);

    trace!("{:?}", cmd);

    bus.renderer.draw_rectangle(Rect {
        left: cmd.xh() as f32 / 4.0,
        right: cmd.xl() as f32 / 4.0,
        top: cmd.yh() as f32 / 4.0,
        bottom: cmd.yl() as f32 / 4.0,
    });
}

#[bitfield(u64)]
struct FillRectangle {
    #[bits(12)]
    yh: u32,
    #[bits(12)]
    xh: u32,
    #[bits(8)]
    __: u64,
    #[bits(12)]
    yl: u32,
    #[bits(12)]
    xl: u32,
    #[bits(8)]
    __: u64,
}
