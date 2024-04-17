use super::renderer::{ColorImage, ColorImageFormat, Rect};
use super::{Bus, Core};
use bitfield_struct::bitfield;
use tracing::{trace, warn};

pub fn set_scissor(_core: &mut Core, bus: Bus, word: u64) {
    let cmd = SetScissor::from(word);

    trace!("{:?}", cmd);

    if cmd.field() {
        warn!("TODO: Set_Scissor interlace suppport");
    }

    bus.renderer.set_scissor(
        bus.gfx,
        bus.rdram,
        Rect {
            left: cmd.xh(),
            right: cmd.xl(),
            top: cmd.yh(),
            bottom: cmd.yl(),
        },
    );
}

pub fn set_color_image(_core: &mut Core, bus: Bus, word: u64) {
    let cmd = SetColorImage::from(word);

    trace!("{:?}", cmd);

    let format = match (cmd.format(), cmd.size()) {
        (Format::ColorIndex, 1) => ColorImageFormat::Index8,
        (Format::Rgba, 2) => ColorImageFormat::Rgba16,
        (Format::Rgba, 3) => ColorImageFormat::Rgba32,
        _ => panic!(
            "Unsupported format for SetColorImage: {:?} {:?}",
            cmd.format(),
            cmd.size(),
        ),
    };

    bus.renderer.set_color_image(
        bus.gfx,
        bus.rdram,
        ColorImage {
            dram_addr: cmd.dram_addr(),
            width: cmd.width() + 1,
            format,
        },
    );
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

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Format {
    Rgba = 0,
    Yuv = 1,
    ColorIndex = 2,
    IA = 3,
    I = 4,
}

impl Format {
    const fn into_bits(self) -> u32 {
        self as u32
    }

    const fn from_bits(value: u32) -> Self {
        match value & 3 {
            0 => Self::Rgba,
            1 => Self::Yuv,
            2 => Self::ColorIndex,
            3 => Self::IA,
            _ => Self::I,
        }
    }
}
