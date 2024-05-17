use super::renderer::Rect;
use super::{Context, Decoder};
use bitfield_struct::bitfield;
use tracing::trace;

pub fn rectangle<const TEXTURE: bool, const FLIP: bool>(
    decoder: &mut Decoder,
    ctx: Context,
    word: u64,
) {
    let cmd = Rectangle::from(word);

    trace!("{:?}", cmd);

    let rect = Rect {
        left: cmd.xh() as f32 / 4.0,
        right: (cmd.xl() as f32 / 4.0),
        top: cmd.yh() as f32 / 4.0,
        bottom: (cmd.yl() as f32 / 4.0),
    };

    trace!("  = {:?}", rect);

    let texture = if TEXTURE {
        let Some(arg) = decoder.commands.pop_front() else {
            decoder.commands.push_front(word);
            decoder.running = false;
            return;
        };

        let coords = TexCoords::from(arg);
        trace!("{:?}", coords);

        let sh = coords.s() as i16 as f32 / 32.0;
        let th = coords.t() as i16 as f32 / 32.0;
        let dsdx = coords.dsdx() as i16 as f32 / 1024.0;
        let dtdy = coords.dtdy() as f32 / 1024.0;

        trace!(
            "SH = {}, TH = {}, DSDX = {}, DTDY={}, FLIP={}",
            sh,
            th,
            dsdx,
            dtdy,
            FLIP
        );

        let sl = sh + (rect.right - rect.left) * dsdx;
        let tl = th + (rect.bottom - rect.top) * dtdy;

        let tex_rect = Rect {
            left: sh,
            right: sl,
            top: th,
            bottom: tl,
        };

        trace!("  = {:?}", tex_rect);
        Some((cmd.tile() as usize, tex_rect, FLIP))
    } else {
        None
    };

    ctx.renderer
        .draw_rectangle(ctx.gfx, ctx.rdram, rect, texture);
}

#[bitfield(u64)]
struct Rectangle {
    #[bits(12)]
    yh: u32,
    #[bits(12)]
    xh: u32,
    #[bits(3)]
    tile: u32,
    #[bits(5)]
    __: u64,
    #[bits(12)]
    yl: u32,
    #[bits(12)]
    xl: u32,
    #[bits(8)]
    __: u64,
}

#[bitfield(u64)]
struct TexCoords {
    dtdy: u16,
    dsdx: u16,
    t: u16,
    s: u16,
}
