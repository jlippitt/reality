use super::Rect;
use crate::gfx::GfxContext;
use crate::rdram::Rdram;
use std::collections::HashMap;
use texture::Texture;
use tracing::trace;

mod texture;

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

#[derive(Clone, Debug, Default)]
pub struct TileDescriptor {
    pub tmem_addr: u32,
    pub width: u32,
    pub format: TextureFormat,
}

#[derive(Clone, Debug, Default)]
pub struct Tile {
    hash_value: u128,
    descriptor: TileDescriptor,
    size: Rect,
}

pub struct Tmem {
    texture_image: TextureImage,
    tmem_data: Vec<u64>,
    tiles: [Tile; 8],
    texture_cache: HashMap<u128, Texture>,
    null_texture: Texture,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl Tmem {
    pub fn new(gfx: &GfxContext) -> Self {
        let bind_group_layout =
            gfx.device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("RDP TMEM Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        Self {
            texture_image: TextureImage::default(),
            tmem_data: vec![0; 512],
            tiles: Default::default(),
            texture_cache: HashMap::new(),
            null_texture: Texture::new(gfx, &bind_group_layout, 1, 1, &[0; 4]),
            bind_group_layout,
        }
    }

    pub fn set_texture_image(&mut self, texture_image: TextureImage) {
        self.texture_image = texture_image;
        trace!("  Texture Image: {:?}", self.texture_image);
    }

    pub fn set_tile(&mut self, tile_id: usize, descriptor: TileDescriptor, hash_value: u64) {
        let tile = &mut self.tiles[tile_id];
        tile.descriptor = descriptor;
        tile.hash_value = (tile.hash_value & 0xffff_ffff_0000_0000) | (hash_value as u128);
        trace!("  Tile {} Descriptor: {:?}", tile_id, tile.descriptor);
        trace!("  Tile {} Hash Value: {:016X}", tile_id, tile.hash_value);
    }

    pub fn load_tile(&mut self, rdram: &Rdram, tile_id: usize, rect: Rect, hash_value: u64) {
        let x_offset = rect.left as usize;
        let x_size = rect.width() as usize;
        let y_offset = rect.top as usize;
        let y_size = rect.height() as usize;

        let tile = &mut self.tiles[tile_id];
        tile.size = rect;
        tile.hash_value = (tile.hash_value & 0x0000_0000_ffff_ffff) | ((hash_value as u128) << 64);
        trace!("  Tile {} Size: {:?}", tile_id, tile.size);
        trace!("  Tile {} Hash Value: {:016X}", tile_id, tile.hash_value);

        let bits_per_pixel = self.texture_image.format.bits_per_pixel();

        let dram_width = (self.texture_image.width as usize * bits_per_pixel + 7) / 8;
        let tmem_width = (x_size * bits_per_pixel + 63) / 64;
        let line_offset = (x_offset * bits_per_pixel + 7) / 8;

        let mut dram_addr = self.texture_image.dram_addr as usize + dram_width * y_offset;
        let mut tmem_addr = tile.descriptor.tmem_addr as usize;

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
            tile.descriptor.tmem_addr,
            tmem_addr,
            tmem_width,
            y_size,
            tmem_width * y_size * 8,
        );
    }

    pub fn get_texture_handle(&mut self, gfx: &GfxContext, tile_id: usize) -> u128 {
        let tile = &self.tiles[tile_id];

        self.texture_cache
            .entry(tile.hash_value)
            .or_insert_with(|| {
                Texture::from_tmem_data(gfx, &self.bind_group_layout, tile, &self.tmem_data)
            });

        tile.hash_value
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self, handle: Option<u128>) -> &wgpu::BindGroup {
        if let Some(hash_value) = handle {
            self.texture_cache.get(&hash_value).unwrap().bind_group()
        } else {
            self.null_texture.bind_group()
        }
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
