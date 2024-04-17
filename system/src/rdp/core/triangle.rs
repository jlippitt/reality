use super::{Bus, Core};
use bitfield_struct::bitfield;
use tracing::{trace, warn};

pub fn triangle<const SHADE: bool, const TEXTURE: bool, const Z_BUFFER: bool>(
    core: &mut Core,
    bus: Bus,
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

    let yh = cmd.yh() as f32 / 4.0;
    let ym = cmd.ym() as f32 / 4.0;
    let yl = cmd.yl() as f32 / 4.0;
    let xh = edge_high.x() as f32 / 65536.0;
    let xl = edge_low.x() as f32 / 65536.0;
    let dxhdy = edge_high.dxdy() as f32 / 65536.0;
    let dxldy = edge_high.dxdy() as f32 / 65536.0;

    let edges: [[f32; 2]; 3] = [
        [xh + (yh - yh.floor()) * dxhdy, yh],
        [xl - 0.25 * dxldy, ym],
        [xh + (yl - yh.floor()) * dxhdy, yl],
    ];

    trace!("  = {:?}", edges);

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

    bus.renderer.draw_triangle(&edges);
}

#[bitfield(u64)]
struct Triangle {
    #[bits(14)]
    yh: i32,
    #[bits(2)]
    __: u64,
    #[bits(14)]
    ym: i32,
    #[bits(2)]
    __: u64,
    #[bits(14)]
    yl: i32,
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
    dxdy: i32,
    #[bits(32)]
    x: i32,
}
