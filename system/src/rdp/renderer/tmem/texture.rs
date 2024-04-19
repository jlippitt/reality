use super::{Format, TextureFormat, Tile};
use crate::gfx::GfxContext;
use tracing::trace;
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
        trace!("{}x{}: {:?}", width, height, data);

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
        let format = tile.descriptor.format;

        let mut buf: [u64; 2048] = [0; 2048];

        let buf_start = deinterleave_tmem_data(
            &mut buf,
            &tmem_data[tile.descriptor.tmem_addr as usize..],
            width,
            height,
            4 << format.1,
        );

        let output = decode_texture(&mut buf, buf_start, format);

        Texture::new(
            gfx,
            bind_group_layout,
            width,
            height,
            &output[0..(width * height * 4) as usize],
        )
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

fn deinterleave_tmem_data(
    buf: &mut [u64],
    tmem_data: &[u64],
    width: u32,
    height: u32,
    bits_per_pixel: usize,
) -> usize {
    let tmem_line_len = ((width as usize * bits_per_pixel) + 63) / 64;
    let buf_start = buf.len() - tmem_line_len * height as usize;

    let mut buf_index = buf_start;
    let mut tmem_index = 0;

    for line in 0..height {
        let tmem_line = &tmem_data[tmem_index..(tmem_index + tmem_line_len)];
        let buf_line = &mut buf[buf_index..(buf_index + tmem_line_len)];

        if (line & 1) != 0 {
            for (dst, src) in buf_line
                .iter_mut()
                .zip(tmem_line.iter().map(|word| (word << 32) | (word >> 32)))
            {
                *dst = src
            }
        } else {
            buf_line.copy_from_slice(tmem_line);
        }

        tmem_index += tmem_line_len;
        buf_index += tmem_line_len;
    }

    buf_start
}

fn decode_texture(buf: &mut [u64], buf_start: usize, format: TextureFormat) -> &[u8] {
    let output_start = match format {
        (Format::Rgba, 3) => buf_start,
        (Format::Rgba, 2) => {
            let read_start = buf_start << 2;

            for index in 0..((buf.len() - buf_start) << 2) {
                let input: &[u16] = bytemuck::must_cast_slice_mut(buf);
                let word = input[read_start + index].swap_bytes();

                let red = ((word >> 11) as u8 & 31) << 3;
                let green = ((word >> 6) as u8 & 31) << 3;
                let blue = ((word >> 1) as u8 & 31) << 3;
                let alpha = (word as u8 & 1) * 255;

                let color = u32::from_le_bytes([red, green, blue, alpha]);

                bytemuck::must_cast_slice_mut::<u64, u32>(buf)[index] = color;
            }

            0
        }
        (Format::I, 1) => {
            let read_start = buf_start << 3;

            for index in 0..((buf.len() - buf_start) << 3) {
                let input: &[u8] = bytemuck::must_cast_slice_mut(buf);
                let intensity = input[read_start + index];
                let color = u32::from_le_bytes([intensity; 4]);
                bytemuck::must_cast_slice_mut::<u64, u32>(buf)[index] = color;
            }

            0
        }
        (Format::I, 0) => {
            let read_start = buf_start << 3;

            for index in 0..((buf.len() - buf_start) << 4) {
                let input: &[u8] = bytemuck::must_cast_slice_mut(buf);
                let word = input[read_start + (index >> 1)];

                let intensity = if (index & 1) == 0 {
                    word & !15
                } else {
                    (word & 15) << 4
                };

                let color = u32::from_le_bytes([intensity; 4]);

                bytemuck::must_cast_slice_mut::<u64, u32>(buf)[index] = color;
            }

            0
        }
        _ => panic!("Unsupported TMEM texture format"),
    };

    bytemuck::must_cast_slice(&buf[output_start..])
}
