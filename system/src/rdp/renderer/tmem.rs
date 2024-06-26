use super::Format;
use super::{Rect, TextureFormat};
use crate::gfx::GfxContext;
use crate::rdram::Rdram;
use std::collections::HashMap;
use std::rc::Rc;
use texture::Texture;
use tile_view::{TileView, TileViewOptions};
use tracing::trace;

mod texture;
mod tile_view;

#[derive(Clone, Debug, Default)]
pub struct TextureImage {
    pub dram_addr: u32,
    pub width: u32,
    pub format: TextureFormat,
}

#[derive(Clone, Debug, Default)]
pub struct TileAddressMode {
    pub clamp: bool,
    pub mirror: bool,
    pub mask: u32,
    pub shift: u32,
}

#[derive(Clone, Debug, Default)]
pub struct TileDescriptor {
    pub tmem_addr: u32,
    pub width: u32,
    pub format: TextureFormat,
    pub palette: u32,
    pub address_s: TileAddressMode,
    pub address_t: TileAddressMode,
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
    tile_view_cache: HashMap<u128, TileView>,
    texture_cache: HashMap<u128, Rc<Texture>>,
    null_tile: TileView,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
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
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let sampler = gfx.device().create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        let null_tile = TileView::new(TileViewOptions {
            gfx,
            sampler: &sampler,
            bind_group_layout: &bind_group_layout,
            texture: Rc::new(Texture::new(gfx, 1, 1, &[0, 0, 0, 255])),
            address_s: &Default::default(),
            address_t: &Default::default(),
        });

        Self {
            texture_image: TextureImage::default(),
            tmem_data: vec![0; 512],
            tiles: Default::default(),
            tile_view_cache: HashMap::new(),
            texture_cache: HashMap::new(),
            null_tile,
            bind_group_layout,
            sampler,
        }
    }

    pub fn set_texture_image(&mut self, texture_image: TextureImage) {
        self.texture_image = texture_image;
        trace!("  Texture Image: {:?}", self.texture_image);
    }

    pub fn set_tile(&mut self, tile_id: usize, descriptor: TileDescriptor, hash_value: u64) {
        let tile = &mut self.tiles[tile_id];
        tile.descriptor = descriptor;
        tile.hash_value = (tile.hash_value & !0xffff_ffff_ffff_ffff) | (hash_value as u128);
        trace!("  Tile {} Descriptor: {:?}", tile_id, tile.descriptor);
        trace!("  Tile {} Hash Value: {:032X}", tile_id, tile.hash_value);
    }

    pub fn tile_size(&self, tile_id: usize) -> &Rect {
        &self.tiles[tile_id].size
    }

    pub fn set_tile_size(&mut self, tile_id: usize, rect: Rect, hash_value: u64) {
        let tile = &mut self.tiles[tile_id];
        tile.size = rect;
        tile.hash_value = (tile.hash_value & 0xffff_ffff_ffff_ffff) | ((hash_value as u128) << 64);
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
        self.tile_view_cache.clear();

        let tile = &self.tiles[tile_id];
        let bits_per_pixel = 4 << self.texture_image.format.1;

        let dram_width = (self.texture_image.width as usize * bits_per_pixel + 7) / 8;
        let dram_line_offset = (x_offset * bits_per_pixel + 7) / 8;
        let tmem_width = (x_size * bits_per_pixel + 63) / 64;

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
        self.tile_view_cache.clear();

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
        y_offset: usize,
        y_delta: usize,
    ) {
        // TODO: Finer-grained cache invalidation
        self.texture_cache.clear();
        self.tile_view_cache.clear();

        let tile = &self.tiles[tile_id];
        let bits_per_pixel = 4 << self.texture_image.format.1;

        let tmem_width = (x_size * bits_per_pixel + 63) / 64;
        let tmem_start = tile.descriptor.tmem_addr as usize;
        let tmem_end = tmem_start + tmem_width;

        let mut dram_addr =
            self.texture_image.dram_addr as usize + (x_offset * bits_per_pixel + 7) / 8;

        if y_delta > 0 {
            dram_addr += y_offset * 2048usize.div_ceil(y_delta) * 8;
        } else if y_offset > 0 {
            panic!("LoadBlock: y_delta of 0 with y_offset {} (greater than 0) would result in infinite DRAM address", y_offset);
        }

        let mut tmem_addr = tmem_start;
        let mut y_pos = 0;

        while tmem_addr < tmem_end {
            let mut tmem_line_start = tmem_addr;

            while (y_pos & 0x0800) == 0 && tmem_addr < tmem_end {
                y_pos += y_delta;
                tmem_addr += 1;
            }

            let dst = &mut self.tmem_data[tmem_line_start..tmem_addr];
            rdram.read_block(dram_addr, dst);
            dram_addr += (tmem_addr - tmem_line_start) << 3;

            if tmem_addr >= tmem_end {
                break;
            }

            tmem_line_start = tmem_addr;

            while (y_pos & 0x0800) != 0 && tmem_addr < tmem_end {
                y_pos += y_delta;
                tmem_addr += 1;
            }

            let dst = &mut self.tmem_data[tmem_line_start..tmem_addr];
            rdram.read_block(dram_addr, dst);
            dram_addr += (tmem_addr - tmem_line_start) << 3;

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

    pub fn get_texture_handle(&mut self, gfx: &GfxContext, tile_id: usize) -> Option<u128> {
        let tile = &self.tiles[tile_id];

        if tile.size.width() < 1.0 || tile.size.height() < 1.0 {
            return None;
        }

        let create_tile_view = || {
            let texture = self
                .texture_cache
                .entry(tile.hash_value & !0x000f_ffff)
                .or_insert_with(|| Rc::new(Texture::from_tmem_data(gfx, tile, &self.tmem_data)));

            TileView::new(TileViewOptions {
                gfx,
                sampler: &self.sampler,
                bind_group_layout: &self.bind_group_layout,
                texture: texture.clone(),
                address_s: &tile.descriptor.address_s,
                address_t: &tile.descriptor.address_t,
            })
        };

        self.tile_view_cache
            .entry(tile.hash_value)
            .or_insert_with(create_tile_view);

        Some(tile.hash_value)
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self, handle: Option<u128>) -> &wgpu::BindGroup {
        if let Some(hash_value) = handle {
            if let Some(tile_view) = self.tile_view_cache.get(&hash_value) {
                return tile_view.bind_group();
            }

            panic!(
                "Tile view with handle {:032X} should be in cache",
                hash_value
            );
        }

        self.null_tile.bind_group()
    }
}
