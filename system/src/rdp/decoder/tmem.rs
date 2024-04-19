use super::renderer::{Rect, TextureFormat, TextureImage, TileDescriptor};
use super::{Context, Decoder, Format};
use bitfield_struct::bitfield;
use tracing::trace;

pub fn set_texture_image(_decoder: &mut Decoder, ctx: Context, word: u64) {
    let cmd = SetTextureImage::from(word);

    trace!("{:?}", cmd);

    ctx.renderer.set_texture_image(TextureImage {
        dram_addr: cmd.dram_addr(),
        width: cmd.width() + 1,
        format: texture_format(cmd.format(), cmd.size()),
    });
}

pub fn set_tile(_decoder: &mut Decoder, ctx: Context, word: u64) {
    let cmd = SetTile::from(word);

    trace!("{:?}", cmd);

    ctx.renderer.set_tile(
        cmd.tile() as usize,
        TileDescriptor {
            tmem_addr: cmd.tmem_addr(),
            width: cmd.line(),
            format: texture_format(cmd.format(), cmd.size()),
            // TODO: The rest
        },
        word & 0x00fb_ffff_00ff_ffff,
    );
}

pub fn set_tile_size(_decoder: &mut Decoder, ctx: Context, word: u64) {
    let cmd = SetTileSize::from(word);

    trace!("{:?}", cmd);

    let rect = Rect {
        left: cmd.sl() as f32 / 4.0,
        right: cmd.sh() as f32 / 4.0 + 1.0,
        top: cmd.tl() as f32 / 4.0,
        bottom: cmd.th() as f32 / 4.0 + 1.0,
    };

    ctx.renderer
        .set_tile_size(cmd.tile() as usize, rect, word & 0x00ff_ffff_00ff_ffff);
}

pub fn load_tile(_decoder: &mut Decoder, ctx: Context, word: u64) {
    let cmd = LoadTile::from(word);

    trace!("{:?}", cmd);

    let rect = Rect {
        left: cmd.sl() as f32 / 4.0,
        right: cmd.sh() as f32 / 4.0 + 1.0,
        top: cmd.tl() as f32 / 4.0,
        bottom: cmd.th() as f32 / 4.0 + 1.0,
    };

    ctx.renderer.load_tile(
        ctx.gfx,
        ctx.rdram,
        cmd.tile() as usize,
        rect,
        word & 0x00ff_ffff_00ff_ffff,
    );
}

fn texture_format(format: Format, size: u32) -> TextureFormat {
    match (format, size) {
        (Format::Rgba, 2) => TextureFormat::Rgba16,
        (Format::Rgba, 3) => TextureFormat::Rgba32,
        (Format::Yuv, 2) => TextureFormat::Yuv16,
        (Format::ColorIndex, 0) => TextureFormat::ClrIndex4,
        (Format::ColorIndex, 1) => TextureFormat::ClrIndex8,
        (Format::IA, 0) => TextureFormat::IA4,
        (Format::IA, 1) => TextureFormat::IA8,
        (Format::IA, 2) => TextureFormat::IA16,
        (Format::I, 0) => TextureFormat::I4,
        (Format::I, 1) => TextureFormat::I8,
        _ => panic!(
            "Unsupported format for SetTextureImage: {:?} {:?}",
            format, size,
        ),
    }
}

#[bitfield(u64)]
struct SetTextureImage {
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
struct SetTile {
    #[bits(4)]
    shift_s: u32,
    #[bits(4)]
    mask_s: u32,
    mirror_s: bool,
    clamp_s: bool,
    #[bits(4)]
    shift_t: u32,
    #[bits(4)]
    mask_t: u32,
    mirror_t: bool,
    clamp_t: bool,
    #[bits(4)]
    palette: u32,
    #[bits(3)]
    tile: u32,
    #[bits(5)]
    __: u64,
    #[bits(9)]
    tmem_addr: u32,
    #[bits(9)]
    line: u32,
    __: bool,
    #[bits(2)]
    size: u32,
    #[bits(3)]
    format: Format,
    #[bits(8)]
    __: u64,
}

#[bitfield(u64)]
struct SetTileSize {
    #[bits(12)]
    sh: u32,
    #[bits(12)]
    th: u32,
    #[bits(3)]
    tile: u64,
    #[bits(5)]
    __: u64,
    #[bits(12)]
    sl: u32,
    #[bits(12)]
    tl: u32,
    #[bits(8)]
    __: u64,
}

#[bitfield(u64)]
struct LoadTile {
    #[bits(12)]
    sh: u32,
    #[bits(12)]
    th: u32,
    #[bits(3)]
    tile: u64,
    #[bits(5)]
    __: u64,
    #[bits(12)]
    sl: u32,
    #[bits(12)]
    tl: u32,
    #[bits(8)]
    __: u64,
}
