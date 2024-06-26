use super::{Context, Decoder};
use bitfield_struct::bitfield;
use std::array;
use tracing::trace;

pub fn triangle<const SHADE: bool, const TEXTURE: bool, const Z_BUFFER: bool>(
    decoder: &mut Decoder,
    ctx: Context,
    word: u64,
) {
    let cmd = Triangle::from(word);
    trace!("{:?}", cmd);

    let param_size = 3 + (SHADE as usize * 8) + (TEXTURE as usize * 8) + (Z_BUFFER as usize * 2);

    // Check we have enough command data to satisfy our parameters,
    // or else we have to wait for more to be uploaded
    if decoder.commands.len() < param_size {
        decoder.commands.push_front(word);
        decoder.running = false;
        return;
    }

    let edge_low = Edge::from(decoder.commands.pop_front().unwrap());
    let edge_high = Edge::from(decoder.commands.pop_front().unwrap());
    let edge_mid = Edge::from(decoder.commands.pop_front().unwrap());
    trace!("L {:?}", edge_low);
    trace!("H {:?}", edge_high);
    trace!("M {:?}", edge_mid);

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
        let shade = Color::from(decoder.commands.pop_front().unwrap());
        let shade_dx = Color::from(decoder.commands.pop_front().unwrap());
        let shade_frac = Color::from(decoder.commands.pop_front().unwrap());
        let shade_frac_dx = Color::from(decoder.commands.pop_front().unwrap());
        let shade_de = Color::from(decoder.commands.pop_front().unwrap());
        let shade_dy = Color::from(decoder.commands.pop_front().unwrap());
        let shade_frac_de = Color::from(decoder.commands.pop_front().unwrap());
        let shade_frac_dy = Color::from(decoder.commands.pop_front().unwrap());
        trace!("Shade: {:?}", shade);
        trace!("Shade DX: {:?}", shade_dx);
        trace!("Shade Frac: {:?}", shade_frac);
        trace!("Shade Frac DY: {:?}", shade_frac_dx);
        trace!("Shade DE: {:?}", shade_de);
        trace!("Shade DX: {:?}", shade_dy);
        trace!("Shade Frac DE: {:?}", shade_frac_de);
        trace!("Shade Frac DY: {:?}", shade_frac_dy);

        let base_color = decode_color(shade, shade_frac);
        let color_dx = decode_color(shade_dx, shade_frac_dx);
        let color_de = decode_color(shade_de, shade_frac_de);
        let color_dy = decode_color(shade_dy, shade_frac_dy);
        trace!("Base Color: {:?}", base_color);
        trace!("Color DX: {:?}", color_dx);
        trace!("Color DE: {:?}", color_de);
        trace!("Color DY: {:?}", color_dy);

        let colors: [[f32; 4]; 3] = [
            array::from_fn(|i| (base_color[i] + high_y * color_de[i]) / 255.0),
            array::from_fn(|i| (base_color[i] + mid_y * color_de[i] + mid_x * color_dx[i]) / 255.0),
            array::from_fn(|i| (base_color[i] + low_y * color_de[i]) / 255.0),
        ];

        trace!("  = {:?}", colors);
        colors
    } else {
        [[0.0; 4]; 3]
    };

    let texture = if TEXTURE {
        let coord = TexCoord::from(decoder.commands.pop_front().unwrap());
        let coord_dx = TexCoord::from(decoder.commands.pop_front().unwrap());
        let coord_frac = TexCoord::from(decoder.commands.pop_front().unwrap());
        let coord_frac_dx = TexCoord::from(decoder.commands.pop_front().unwrap());
        let coord_de = TexCoord::from(decoder.commands.pop_front().unwrap());
        let coord_dy = TexCoord::from(decoder.commands.pop_front().unwrap());
        let coord_frac_de = TexCoord::from(decoder.commands.pop_front().unwrap());
        let coord_frac_dy = TexCoord::from(decoder.commands.pop_front().unwrap());
        trace!("Texture: {:?}", coord);
        trace!("Texture DX: {:?}", coord_dx);
        trace!("Texture Frac: {:?}", coord_frac);
        trace!("Texture Frac DY: {:?}", coord_frac_dx);
        trace!("Texture DE: {:?}", coord_de);
        trace!("Texture DX: {:?}", coord_dy);
        trace!("Texture Frac DE: {:?}", coord_frac_de);
        trace!("Texture Frac DY: {:?}", coord_frac_dy);

        let base_texel = decode_tex_coord(coord, coord_frac);
        let texel_dx = decode_tex_coord(coord_dx, coord_frac_dx);
        let texel_de = decode_tex_coord(coord_de, coord_frac_de);
        let texel_dy = decode_tex_coord(coord_dy, coord_frac_dy);
        trace!("Base Texel: {:?}", base_texel);
        trace!("Texel DX: {:?}", texel_dx);
        trace!("Texel DE: {:?}", texel_de);
        trace!("Texel DY: {:?}", texel_dy);

        let tex_coords: [[f32; 3]; 3] = [
            array::from_fn(|i| base_texel[i] + high_y * texel_de[i]),
            array::from_fn(|i| base_texel[i] + mid_y * texel_de[i] + mid_x * texel_dx[i]),
            array::from_fn(|i| base_texel[i] + low_y * texel_de[i]),
        ];

        trace!("  = {:?}", tex_coords);
        Some((cmd.tile() as usize, tex_coords))
    } else {
        None
    };

    let z_values: [f32; 3] = if Z_BUFFER {
        let z_dzdx_word = decoder.commands.pop_front().unwrap();
        let dzde_dzdy_word = decoder.commands.pop_front().unwrap();

        let z = ((z_dzdx_word >> 32) as u32 as i32) as f32 / 65536.0;
        let dzdx = (z_dzdx_word as u32 as i32) as f32 / 65536.0;
        let dzde = ((dzde_dzdy_word >> 32) as u32 as i32) as f32 / 65536.0;
        let dzdy = (dzde_dzdy_word as u32 as i32) as f32 / 65536.0;
        trace!("Z: {}, DZDX: {}, DZDE: {}, DZDY: {}", z, dzdx, dzde, dzdy);

        let z_values = [
            (z + high_y * dzde) / 32768.0,
            (z + mid_y * dzde + mid_x * dzdx) / 32768.0,
            (z + low_y * dzde) / 32768.0,
        ];

        trace!("  = {:?}", z_values);
        z_values
    } else {
        [0.0; 3]
    };

    ctx.renderer
        .draw_triangle(ctx.gfx, ctx.rdram, edges, colors, texture, z_values);
}

fn decode_color(integer: Color, fraction: Color) -> [f32; 4] {
    [
        ((integer.r() << 16) | fraction.r()) as i32 as f32 / 65536.0,
        ((integer.g() << 16) | fraction.g()) as i32 as f32 / 65536.0,
        ((integer.b() << 16) | fraction.b()) as i32 as f32 / 65536.0,
        ((integer.a() << 16) | fraction.a()) as i32 as f32 / 65536.0,
    ]
}

fn decode_tex_coord(integer: TexCoord, fraction: TexCoord) -> [f32; 3] {
    [
        ((integer.s() << 16) | fraction.s()) as i32 as f32 / 65536.0 / 32.0,
        ((integer.t() << 16) | fraction.t()) as i32 as f32 / 65536.0 / 32.0,
        ((integer.w() << 16) | fraction.w()) as i32 as f32 / 65536.0 / 32.0,
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
    a: u32,
    #[bits(16)]
    b: u32,
    #[bits(16)]
    g: u32,
    #[bits(16)]
    r: u32,
}

#[bitfield(u64)]
struct TexCoord {
    #[bits(16)]
    __: u32,
    #[bits(16)]
    w: u32,
    #[bits(16)]
    t: u32,
    #[bits(16)]
    s: u32,
}
