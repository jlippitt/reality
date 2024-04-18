use super::Rect;
use crate::rdram::Rdram;
use tracing::trace;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum TextureFormat {
    Rgba16,
    #[default]
    Rgba32,
    Yuv16,
    ClrIndex4,
    ClrIndex8,
    IA4,
    IA8,
    IA16,
    I4,
    I8,
}

#[derive(Clone, Debug, Default)]
pub struct TextureImage {
    pub dram_addr: u32,
    pub width: u32,
    pub format: TextureFormat,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TileDescriptor {
    pub tmem_addr: u32,
    pub width: u32,
    pub format: TextureFormat,
}

pub struct Tmem {
    texture_image: TextureImage,
    tmem_data: Vec<u64>,
    tile_descriptors: [TileDescriptor; 8],
    tile_sizes: [Rect; 8],
}

impl Tmem {
    pub fn new() -> Self {
        Self {
            texture_image: TextureImage::default(),
            tmem_data: vec![0; 512],
            tile_descriptors: Default::default(),
            tile_sizes: Default::default(),
        }
    }

    pub fn set_texture_image(&mut self, texture_image: TextureImage) {
        self.texture_image = texture_image;
        trace!("  Texture Image: {:?}", self.texture_image);
    }

    pub fn set_tile(&mut self, index: usize, tile: TileDescriptor) {
        self.tile_descriptors[index] = tile;
        trace!(
            "  Tile {} Descriptor: {:?}",
            index,
            self.tile_descriptors[index]
        );
    }

    pub fn load_tile(&mut self, rdram: &Rdram, index: usize, rect: Rect) {
        let x_offset = rect.left as usize;
        let x_size = rect.width() as usize;
        let y_offset = rect.top as usize;
        let y_size = rect.height() as usize;

        self.tile_sizes[index] = rect;
        trace!("  Tile {} Size: {:?}", index, self.tile_sizes[index]);

        let bits_per_pixel = self.texture_image.format.bits_per_pixel();

        let dram_width = (self.texture_image.width as usize * bits_per_pixel + 7) / 8;
        let tmem_width = (x_size * bits_per_pixel + 63) / 64;
        let line_offset = (x_offset * bits_per_pixel + 7) / 8;

        let mut dram_addr = self.texture_image.dram_addr as usize + dram_width * y_offset;
        let mut tmem_addr = self.tile_descriptors[index].tmem_addr as usize;

        for line in 0..y_size {
            let dst = &mut self.tmem_data[tmem_addr..(tmem_addr + tmem_width)];
            rdram.read_block(dram_addr + line_offset, dst);

            if (line & 1) != 0 {
                for word in dst {
                    *word = (*word << 32) | (*word >> 32);
                }
            }

            dram_addr += dram_width;
            tmem_addr += tmem_width;
        }

        trace!(
            "  Tile data uploaded from {:08X}..{:08X} to {:04X}..{:04X} ({}x{} words = {} bytes)",
            self.texture_image.dram_addr,
            dram_addr,
            self.tile_descriptors[index].tmem_addr,
            tmem_addr,
            tmem_width,
            y_size,
            tmem_width * y_size * 8,
        );
    }
}

impl TextureFormat {
    fn bits_per_pixel(self) -> usize {
        match self {
            Self::Rgba16 => 16,
            Self::Rgba32 => 32,
            Self::Yuv16 => 16,
            Self::ClrIndex4 => 4,
            Self::ClrIndex8 => 8,
            Self::IA4 => 4,
            Self::IA8 => 8,
            Self::IA16 => 16,
            Self::I4 => 4,
            Self::I8 => 8,
        }
    }
}
