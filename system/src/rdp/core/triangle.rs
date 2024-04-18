use super::{Bus, Core};
use bitfield_struct::bitfield;
use std::array;
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

    let high_y = yh - yh.floor();
    let mid_y = ym - yh.floor();
    let mid_x = xl - (xh + mid_y * dxhdy);
    let low_y = yl - yh.floor();

    let edges: [[f32; 2]; 3] = [
        [xh + high_y * dxhdy, yh],
        [xl, ym],
        [xh + low_y * dxhdy, yl],
    ];

    trace!("  = {:?}", edges);

    let colors: [[f32; 4]; 3] = if SHADE {
        let shade = Color::from(core.commands.pop_front().unwrap());
        let shade_dx = Color::from(core.commands.pop_front().unwrap());
        let shade_frac = Color::from(core.commands.pop_front().unwrap());
        let shade_frac_dx = Color::from(core.commands.pop_front().unwrap());
        let shade_de = Color::from(core.commands.pop_front().unwrap());
        let shade_dy = Color::from(core.commands.pop_front().unwrap());
        let shade_frac_de = Color::from(core.commands.pop_front().unwrap());
        let shade_frac_dy = Color::from(core.commands.pop_front().unwrap());

        let base_color = decode_color(shade, shade_frac);
        let color_dx = decode_color(shade_dx, shade_frac_dx);
        let color_de = decode_color(shade_de, shade_frac_de);
        let color_dy = decode_color(shade_dy, shade_frac_dy);
        trace!("Base Color: {:?}", base_color);
        trace!("Color DX: {:?}", color_dx);
        trace!("Color DE: {:?}", color_de);
        trace!("Color DY: {:?}", color_dy);

        let colors: [[f32; 4]; 3] = [
            array::from_fn(|i| base_color[i] + high_y * color_de[i]),
            array::from_fn(|i| base_color[i] + mid_y * color_de[i] + mid_x * color_dx[i]),
            array::from_fn(|i| base_color[i] + low_y * color_de[i]),
        ];

        trace!("  = {:?}", colors);
        colors
    } else {
        [bus.renderer.blend_color(); 3]
    };

    if TEXTURE {
        for _ in 0..8 {
            core.commands.pop_front().unwrap();
        }

        warn!("TODO: Textured triangles");
    }

    // TODO: If z_source_sel is '1', do we ignore all this?
    let z_values: [f32; 3] = if Z_BUFFER {
        let z_dzdx_word = core.commands.pop_front().unwrap();
        let dzde_dzdy_word = core.commands.pop_front().unwrap();

        let z = ((z_dzdx_word >> 32) as i32) as f32 / 65536.0;
        let dzdx = (z_dzdx_word as i32) as f32 / 65536.0;
        let dzde = ((dzde_dzdy_word >> 32) as i32) as f32 / 65536.0;
        let dzdy = (dzde_dzdy_word as i32) as f32 / 65536.0;
        trace!("Z: {}, DZDX: {}, DZDE: {}, DZDY: {}", z, dzdx, dzde, dzdy);

        let z_values = [
            z + high_y * dzde,
            z + mid_y * dzde + mid_x * dzdx,
            z + low_y * dzde,
        ];

        trace!("  = {:?}", z_values);
        z_values
    } else {
        // Assume we don't use prim_depth here?
        [0.0; 3]
    };

    bus.renderer.draw_triangle(edges, colors, z_values);
}

fn decode_color(integer: Color, fraction: Color) -> [f32; 4] {
    [
        ((integer.r() << 16) | fraction.r()) as f32 / 65536.0,
        ((integer.g() << 16) | fraction.g()) as f32 / 65536.0,
        ((integer.b() << 16) | fraction.b()) as f32 / 65536.0,
        ((integer.a() << 16) | fraction.a()) as f32 / 65536.0,
    ]
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
    #[bits(30)]
    dxdy: i32,
    #[bits(2)]
    __: u64,
    #[bits(30)]
    x: i32,
    #[bits(2)]
    __: u64,
}

#[bitfield(u64)]
struct Color {
    #[bits(16)]
    a: i32,
    #[bits(16)]
    b: i32,
    #[bits(16)]
    g: i32,
    #[bits(16)]
    r: i32,
}
