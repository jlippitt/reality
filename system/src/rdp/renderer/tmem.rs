use super::Format;
use super::{Rect, TextureFormat};
use crate::gfx::GfxContext;
use crate::rdram::Rdram;
use std::collections::HashMap;
use texture::Texture;
use tracing::trace;

mod texture;

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
    pub palette: u32,
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
        trace!("  Tile {} Hash Value: {:032X}", tile_id, tile.hash_value);
    }

    pub fn set_tile_size(&mut self, tile_id: usize, rect: Rect, hash_value: u64) {
        let tile = &mut self.tiles[tile_id];
        tile.size = rect;
        tile.hash_value = (tile.hash_value & 0x0000_0000_ffff_ffff) | ((hash_value as u128) << 64);
        trace!("  Tile {} Size: {:?}", tile_id, tile.size);
        trace!("  Tile {} Hash Value: {:032X}", tile_id, tile.hash_value);
    }

    pub fn load_tile(
        &mut self,
        rdram: &Rdram,
        tile_id: usize,
        x_offset: usize,
        x_size: usize,
        y_offset: usize,
        y_size: usize,
    ) {
        // TODO: Finer-grained cache invalidation
        self.texture_cache.clear();

        let tile = &self.tiles[tile_id];
        let bits_per_pixel = 4 << self.texture_image.format.1;

        let dram_width = (self.texture_image.width as usize * bits_per_pixel + 7) / 8;
        let dram_line_offset = (x_offset * bits_per_pixel + 7) / 8;
        let tmem_width = ((x_size * bits_per_pixel + 63) / 8).min(dram_width) / 8;

        let mut dram_addr = self.texture_image.dram_addr as usize + dram_width * y_offset;
        let mut tmem_addr = tile.descriptor.tmem_addr as usize;

        for line in 0..y_size {
            let dst = &mut self.tmem_data[tmem_addr..(tmem_addr + tmem_width)];
            rdram.read_block(dram_addr + dram_line_offset, dst);

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

    pub fn load_tlut(
        &mut self,
        rdram: &Rdram,
        tile_id: usize,
        x_offset: usize,
        x_size: usize,
        y_offset: usize,
        y_size: usize,
    ) {
        // TODO: Finer-grained cache invalidation
        self.texture_cache.clear();

        // TODO: Load TLUT
        let tile = &self.tiles[tile_id];
        let bits_per_pixel = 4 << self.texture_image.format.1;

        let dram_width = (self.texture_image.width as usize * bits_per_pixel + 7) / 8;
        let dram_line_offset = (x_offset * bits_per_pixel + 7) / 8;

        let mut dram_addr =
            self.texture_image.dram_addr as usize + y_offset * dram_width + dram_line_offset;
        let mut tmem_addr = tile.descriptor.tmem_addr as usize;

        for _ in 0..(x_size * y_size) {
            let color = rdram.read_single::<u16>(dram_addr).swap_bytes();

            let word = ((color as u64) << 48)
                | ((color as u64) << 32)
                | ((color as u64) << 16)
                | (color as u64);

            self.tmem_data[tmem_addr] = word;
            tmem_addr += 1;
            dram_addr += 2;
        }

        trace!(
            "  TLUT data uploaded from {:08X}..{:08X} to {:04X}..{:04X} ({}x{} words = {} bytes)",
            self.texture_image.dram_addr,
            dram_addr,
            tile.descriptor.tmem_addr,
            tmem_addr,
            x_size,
            y_size,
            x_size * y_size * 2,
        );
    }

    pub fn load_block(
        &mut self,
        rdram: &Rdram,
        tile_id: usize,
        x_offset: usize,
        x_size: usize,
        _y_offset: usize,
        y_delta: usize,
    ) {
        // TODO: Finer-grained cache invalidation
        self.texture_cache.clear();

        let tile = &self.tiles[tile_id];
        let bits_per_pixel = 4 << self.texture_image.format.1;

        let dram_line_offset = (x_offset * bits_per_pixel + 7) / 8;
        let tmem_width = (x_size * bits_per_pixel + 63) / 64;

        // TODO: Does y_offset ('tl') get used at all?
        let mut dram_addr = self.texture_image.dram_addr as usize + dram_line_offset;
        let mut tmem_addr = tile.descriptor.tmem_addr as usize;

        let mut y_pos = 0;

        while tmem_addr < tmem_width {
            let mut tmem_start = tmem_addr;

            while (y_pos & 0x0800) == 0 && tmem_addr < tmem_width {
                y_pos += y_delta;
                tmem_addr += 1;
            }

            let dst = &mut self.tmem_data[tmem_start..tmem_addr];
            rdram.read_block(dram_addr, dst);
            dram_addr += (tmem_addr - tmem_start) << 3;

            if tmem_addr >= tmem_width {
                break;
            }

            tmem_start = tmem_addr;

            while (y_pos & 0x0800) != 0 && tmem_addr < tmem_width {
                y_pos += y_delta;
                tmem_addr += 1;
            }

            let dst = &mut self.tmem_data[tmem_start..tmem_addr];
            rdram.read_block(dram_addr, dst);
            dram_addr += (tmem_addr - tmem_start) << 3;

            for word in dst {
                *word = (*word << 32) | (*word >> 32);
            }
        }

        trace!(
            "  Block data uploaded from {:08X}..{:08X} to {:04X}..{:04X} ({} words = {} bytes)",
            self.texture_image.dram_addr,
            dram_addr,
            tile.descriptor.tmem_addr,
            tmem_addr,
            tmem_addr - tile.descriptor.tmem_addr as usize,
            dram_addr - self.texture_image.dram_addr as usize,
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
