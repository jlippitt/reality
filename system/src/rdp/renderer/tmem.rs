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
    tile_descriptors: [TileDescriptor; 8],
}

impl Tmem {
    pub fn new() -> Self {
        Self {
            texture_image: TextureImage::default(),
            tile_descriptors: Default::default(),
        }
    }

    pub fn set_texture_image(&mut self, texture_image: TextureImage) {
        self.texture_image = texture_image;
        trace!("  Texture Image: {:?}", self.texture_image);
    }

    pub fn set_tile(&mut self, index: usize, tile: TileDescriptor) {
        self.tile_descriptors[index] = tile;
        trace!("  Tile {}: {:?}", index, self.tile_descriptors[index]);
    }
}
