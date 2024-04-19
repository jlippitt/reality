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

        let mut buf: [u64; 4096] = [0; 4096];

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
    println!("{}x{} = {}, {}", width, height, buf_start, tmem_line_len);

    let mut buf_index = buf_start;
    let mut tmem_index = 0;

    for line in 0..(height as usize).min(tmem_data.len() / tmem_line_len) {
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
        (Format::Rgba, 2) => decode_color16(buf, buf_start, |word| {
            let red = ((word >> 11) as u8 & 31) << 3;
            let green = ((word >> 6) as u8 & 31) << 3;
            let blue = ((word >> 1) as u8 & 31) << 3;
            let alpha = (word as u8 & 1) * 255;
            u32::from_le_bytes([red, green, blue, alpha])
        }),
        (Format::IA, 2) => decode_color16(buf, buf_start, |word| {
            let intensity = (word >> 8) as u8;
            let alpha = word as u8;
            u32::from_le_bytes([intensity, intensity, intensity, alpha])
        }),
        (Format::IA, 1) => decode_color8(buf, buf_start, |word| {
            let intensity = word & 0xf0;
            let alpha = word << 4;
            u32::from_le_bytes([intensity, intensity, intensity, alpha])
        }),
        (Format::IA, 0) => decode_color4(buf, buf_start, |word| {
            (
                {
                    let intensity = word & !31;
                    let alpha = ((word >> 4) & 1) * 255;
                    u32::from_le_bytes([intensity, intensity, intensity, alpha])
                },
                {
                    let intensity = (word & 14) << 4;
                    let alpha = (word & 1) * 255;
                    u32::from_le_bytes([intensity, intensity, intensity, alpha])
                },
            )
        }),
        (Format::I, 1) => decode_color8(buf, buf_start, |word| u32::from_le_bytes([word; 4])),
        (Format::I, 0) => decode_color4(buf, buf_start, |word| {
            (
                {
                    let intensity = word & !15;
                    u32::from_le_bytes([intensity; 4])
                },
                {
                    let intensity = (word & 15) << 4;
                    u32::from_le_bytes([intensity; 4])
                },
            )
        }),
        _ => panic!("Unsupported TMEM texture format"),
    };

    bytemuck::must_cast_slice(&buf[output_start..])
}

fn decode_color4(buf: &mut [u64], buf_start: usize, cb: impl Fn(u8) -> (u32, u32)) -> usize {
    let read_start = buf_start << 3;

    for index in 0..((buf.len() - buf_start) << 3) {
        let input: &[u8] = bytemuck::must_cast_slice_mut(buf);
        let word = input[read_start + index];
        let color = cb(word);
        let output: &mut [u32] = bytemuck::must_cast_slice_mut(buf);
        output[index << 1] = color.0;
        output[(index << 1) | 1] = color.1;
    }

    0
}

fn decode_color8(buf: &mut [u64], buf_start: usize, cb: impl Fn(u8) -> u32) -> usize {
    let read_start = buf_start << 3;

    for index in 0..((buf.len() - buf_start) << 3) {
        let input: &[u8] = bytemuck::must_cast_slice_mut(buf);
        let word = input[read_start + index];
        let color = cb(word);
        bytemuck::must_cast_slice_mut::<u64, u32>(buf)[index] = color;
    }

    0
}

fn decode_color16(buf: &mut [u64], buf_start: usize, cb: impl Fn(u16) -> u32) -> usize {
    let read_start = buf_start << 2;

    for index in 0..((buf.len() - buf_start) << 2) {
        let input: &[u16] = bytemuck::must_cast_slice_mut(buf);
        let word = input[read_start + index].swap_bytes();
        let color = cb(word);
        bytemuck::must_cast_slice_mut::<u64, u32>(buf)[index] = color;
    }

    0
}
