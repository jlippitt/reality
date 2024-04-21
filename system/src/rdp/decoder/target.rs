use super::renderer::{ColorImage, Format, Rect};
use super::{Context, Decoder};
use bitfield_struct::bitfield;
use tracing::{trace, warn};

pub fn set_scissor(_decoder: &mut Decoder, ctx: Context, word: u64) {
    let cmd = SetScissor::from(word);

    trace!("{:?}", cmd);

    if cmd.field() {
        warn!("TODO: Set_Scissor interlace suppport");
    }

    ctx.renderer.set_scissor(
        ctx.gfx,
        ctx.rdram,
        Rect {
            left: cmd.xh() as f32 / 4.0,
            right: cmd.xl() as f32 / 4.0,
            top: cmd.yh() as f32 / 4.0,
            bottom: cmd.yl() as f32 / 4.0,
        },
    );
}

pub fn set_color_image(_decoder: &mut Decoder, ctx: Context, word: u64) {
    let cmd = SetColorImage::from(word);

    trace!("{:?}", cmd);

    ctx.renderer.set_color_image(
        ctx.gfx,
        ctx.rdram,
        ColorImage {
            dram_addr: cmd.dram_addr(),
            width: cmd.width() + 1,
            format: (cmd.format(), cmd.size()),
        },
    );
}

pub fn set_fill_color(_decoder: &mut Decoder, ctx: Context, word: u64) {
    let cmd = SetFillColor::from(word);
    trace!("{:?}", cmd);
    ctx.renderer
        .set_fill_color(ctx.gfx, ctx.rdram, cmd.packed_color());
}

#[bitfield(u64)]
struct SetScissor {
    #[bits(12)]
    yl: u32,
    #[bits(12)]
    xl: u32,
    odd_line: bool,
    field: bool,
    #[bits(6)]
    __: u64,
    #[bits(12)]
    yh: u32,
    #[bits(12)]
    xh: u32,
    #[bits(8)]
    __: u64,
}

#[bitfield(u64)]
struct SetColorImage {
    #[bits(26)]
    dram_addr: u32,
    #[bits(6)]
    __: u64,
    #[bits(10)]
    width: u32,
    #[bits(9)]
    __: u64,
    #[bits(2)]
    size: u32,
    #[bits(3)]
    format: Format,
    #[bits(8)]
    __: u64,
}

#[bitfield(u64)]
struct SetFillColor {
    packed_color: u32,
    __: u32,
}
