use super::renderer::Rect;
use super::{Bus, Core};
use bitfield_struct::bitfield;
use tracing::trace;

pub fn rectangle<const TEXTURE: bool, const FLIP: bool>(core: &mut Core, bus: Bus, word: u64) {
    let cmd = Rectangle::from(word);

    trace!("{:?}", cmd);

    let rect = Rect {
        left: cmd.xh() as f32 / 4.0,
        right: cmd.xl() as f32 / 4.0,
        top: cmd.yh() as f32 / 4.0,
        bottom: cmd.yl() as f32 / 4.0,
    };

    trace!("  = {:?}", rect);

    let texture = if TEXTURE {
        let Some(coords) = core.commands.pop_front() else {
            core.commands.push_front(word);
            core.running = false;
            return;
        };

        let sh = (coords >> 48) as i16 as f32 / 32.0;
        let th = (coords >> 32) as i16 as f32 / 32.0;
        let dsdx = (coords >> 16) as i16 as f32 / 1024.0;
        let dtdy = coords as i16 as f32 / 1024.0;

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

    bus.renderer.draw_rectangle(bus.gfx, rect, texture);
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
