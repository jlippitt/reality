use super::renderer::{Format, Rect, TextureImage, TileDescriptor};
use super::{Context, Decoder};
use bitfield_struct::bitfield;
use tracing::trace;

pub fn set_texture_image(_decoder: &mut Decoder, ctx: Context, word: u64) {
    let cmd = SetTextureImage::from(word);

    trace!("{:?}", cmd);

    ctx.renderer.set_texture_image(TextureImage {
        dram_addr: cmd.dram_addr(),
        width: cmd.width() + 1,
        format: (cmd.format(), cmd.size()),
    });
}

pub fn set_tile(_decoder: &mut Decoder, ctx: Context, word: u64) {
    let cmd = SetTile::from(word);

    trace!("{:?}", cmd);

    ctx.renderer.set_tile(
        cmd.tile(),
        TileDescriptor {
            tmem_addr: cmd.tmem_addr(),
            width: cmd.line(),
            format: (cmd.format(), cmd.size()),
            palette: cmd.palette(),
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
        .set_tile_size(cmd.tile(), rect, word & 0x00ff_ffff_00ff_ffff);
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

    ctx.renderer
        .set_tile_size(cmd.tile(), rect, word & 0x00ff_ffff_00ff_ffff);

    ctx.renderer.load_tile(
        ctx.gfx,
        ctx.rdram,
        cmd.tile(),
        cmd.sl() / 4,
        ((cmd.sh() - cmd.sl()) / 4) + 1,
        cmd.tl() / 4,
        ((cmd.th() - cmd.tl()) / 4) + 1,
    );
}

pub fn load_tlut(_decoder: &mut Decoder, ctx: Context, word: u64) {
    let cmd = LoadTlut::from(word);

    trace!("{:?}", cmd);

    ctx.renderer.load_tlut(
        ctx.gfx,
        ctx.rdram,
        cmd.tile(),
        cmd.sl() / 4,
        ((cmd.sh() - cmd.sl()) / 4) + 1,
        cmd.tl() / 4,
        ((cmd.th() - cmd.tl()) / 4) + 1,
    );
}

pub fn load_block(_decoder: &mut Decoder, ctx: Context, word: u64) {
    let cmd = LoadBlock::from(word);

    trace!("{:?}", cmd);

    let rect = Rect {
        left: cmd.sl() as f32,
        right: cmd.sh() as f32 + 1.0,
        top: cmd.tl() as f32,
        bottom: cmd.dxt() as f32 / 2048.0,
    };

    ctx.renderer.set_tile_size(
        cmd.tile(),
        rect,
        // Add an extra bit to specify that tile size was set with LoadBlock
        // command, due to its different parameter format
        (word & 0x00ff_ffff_00ff_ffff) | 0x8000_0000_0000_0000,
    );

    ctx.renderer.load_block(
        ctx.gfx,
        ctx.rdram,
        cmd.tile(),
        cmd.sl(),
        (cmd.sh() - cmd.sl()) + 1,
        cmd.tl(),
        cmd.dxt(),
    );
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
    tile: usize,
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
    th: u32,
    #[bits(12)]
    sh: u32,
    #[bits(3)]
    tile: usize,
    #[bits(5)]
    __: u64,
    #[bits(12)]
    tl: u32,
    #[bits(12)]
    sl: u32,
    #[bits(8)]
    __: u64,
}

#[bitfield(u64)]
struct LoadTile {
    #[bits(12)]
    th: usize,
    #[bits(12)]
    sh: usize,
    #[bits(3)]
    tile: usize,
    #[bits(5)]
    __: u64,
    #[bits(12)]
    tl: usize,
    #[bits(12)]
    sl: usize,
    #[bits(8)]
    __: u64,
}

#[bitfield(u64)]
struct LoadTlut {
    #[bits(12)]
    th: usize,
    #[bits(12)]
    sh: usize,
    #[bits(3)]
    tile: usize,
    #[bits(5)]
    __: u64,
    #[bits(12)]
    tl: usize,
    #[bits(12)]
    sl: usize,
    #[bits(8)]
    __: u64,
}

#[bitfield(u64)]
struct LoadBlock {
    #[bits(12)]
    dxt: usize,
    #[bits(12)]
    sh: usize,
    #[bits(3)]
    tile: usize,
    #[bits(5)]
    __: u64,
    #[bits(12)]
    tl: usize,
    #[bits(12)]
    sl: usize,
    #[bits(8)]
    __: u64,
}
