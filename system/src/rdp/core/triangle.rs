use super::{Bus, Core};
use bitfield_struct::bitfield;
use tracing::{trace, warn};

pub fn triangle<const SHADE: bool, const TEXTURE: bool, const Z_BUFFER: bool>(
    core: &mut Core,
    _bus: Bus,
    word: u64,
) {
    let cmd = Triangle::from(word);
    trace!("{:?}", cmd);

    let param_size = 3 + (SHADE as usize * 8) + (TEXTURE as usize * 8) + (Z_BUFFER as usize * 2);

    // Check we have enough command data to satisfy our parameters,
    // or else we have to wait for more to be uploaded
    if core.commands.len() < param_size {
        core.commands.push_front(word);
        core.running = false;
        return;
    }

    let edge_low = Edge::from(core.commands.pop_front().unwrap());
    let edge_high = Edge::from(core.commands.pop_front().unwrap());
    let edge_mid = Edge::from(core.commands.pop_front().unwrap());
    trace!("{:?}", edge_low);
    trace!("{:?}", edge_high);
    trace!("{:?}", edge_mid);

    if SHADE {
        for _ in 0..8 {
            core.commands.pop_front().unwrap();
        }

        warn!("TODO: Shaded triangles");
    }

    if TEXTURE {
        for _ in 0..8 {
            core.commands.pop_front().unwrap();
        }

        warn!("TODO: Textured triangles");
    }

    if Z_BUFFER {
        for _ in 0..2 {
            core.commands.pop_front().unwrap();
        }

        warn!("TODO: Z-buffer triangles");
    }
}

#[bitfield(u64)]
struct Triangle {
    #[bits(14)]
    yh: u32,
    #[bits(2)]
    __: u64,
    #[bits(14)]
    ym: u32,
    #[bits(2)]
    __: u64,
    #[bits(14)]
    yl: u32,
    #[bits(2)]
    __: u64,
    #[bits(3)]
    tile: u32,
    #[bits(3)]
    level: u32,
    __: bool,
    right: bool,
    #[bits(8)]
    __: u64,
}

#[bitfield(u64)]
struct Edge {
    #[bits(32)]
    dxdy: u32,
    #[bits(32)]
    x: u32,
}
