use crate::gfx::{self, GfxContext};
use tracing::trace;
use wgpu::util::DeviceExt;

const WIDTH: u32 = 4;

pub struct FillColor {
    value: u32,
    texture: wgpu::Texture,
    _view: wgpu::TextureView,
    _sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl FillColor {
    pub fn new(gfx: &GfxContext) -> Self {
        let size = wgpu::Extent3d {
            width: WIDTH,
            height: 1,
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
            &[0; (WIDTH * 4) as usize],
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = gfx.device().create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group_layout =
            gfx.device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("RDP Fill Color Bind Group Layout"),
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

        let bind_group = gfx.device().create_bind_group({
            &wgpu::BindGroupDescriptor {
                label: Some("RDP Fill Color Bind Group"),
                layout: &bind_group_layout,
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
            value: 0,
            texture,
            _view: view,
            _sampler: sampler,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn value(&self) -> u32 {
        self.value
    }

    pub fn set_value(&mut self, value: u32) {
        self.value = value;
        trace!("  Fill Color: {:08X}", value);
    }

    pub fn upload(&mut self, queue: &wgpu::Queue, pixel_size: u32) {
        let colors: [u32; WIDTH as usize] = match pixel_size {
            3 => [self.value.swap_bytes(); 4],
            2 => {
                let color0 = gfx::decode_rgba16((self.value >> 16) as u16);
                let color1 = gfx::decode_rgba16(self.value as u16);
                [color0, color1, color0, color1]
            }
            1 => [
                u32::from_le_bytes([(self.value >> 24) as u8; 4]),
                u32::from_le_bytes([(self.value >> 16) as u8; 4]),
                u32::from_le_bytes([(self.value >> 8) as u8; 4]),
                u32::from_le_bytes([self.value as u8; 4]),
            ],
            _ => [0; 4],
        };

        let data: &[u8] = bytemuck::must_cast_slice(&colors);

        trace!("  Fill Color Upload Data: {:02X?}", data);

        queue.write_texture(
            self.texture.as_image_copy(),
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(WIDTH * 4),
                rows_per_image: Some(1),
            },
            self.texture.size(),
        )
    }
}
