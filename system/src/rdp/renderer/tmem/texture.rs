use super::Tile;
use crate::gfx::GfxContext;
use wgpu::util::DeviceExt;

pub struct Texture {
    _texture: wgpu::Texture,
    _view: wgpu::TextureView,
    _sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
}

impl Texture {
    pub fn new(
        gfx: &GfxContext,
        bind_group_layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = gfx.device().create_texture_with_data(
            gfx.queue(),
            &wgpu::TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            data,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

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

        let bind_group = gfx.device().create_bind_group({
            &wgpu::BindGroupDescriptor {
                label: None,
                layout: bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            }
        });

        Self {
            _texture: texture,
            _view: view,
            _sampler: sampler,
            bind_group,
        }
    }

    pub fn from_tmem_data(
        gfx: &GfxContext,
        bind_group_layout: &wgpu::BindGroupLayout,
        tile: &Tile,
        tmem_data: &[u64],
    ) -> Self {
        let width = tile.size.width() as u32;
        let height = tile.size.height() as u32;

        let data: &[u8] =
            bytemuck::must_cast_slice(&tmem_data[tile.descriptor.tmem_addr as usize..]);

        Texture::new(
            gfx,
            bind_group_layout,
            width,
            height,
            &data[0..(width * height * 4) as usize],
        )
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
