use super::{Format, TextureFormat, Tile};
use crate::gfx::{self, GfxContext};
use tracing::trace;
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct Texture {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl Texture {
    pub fn new(gfx: &GfxContext, width: u32, height: u32, data: &[u8]) -> Self {
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

        Self {
            _texture: texture,
            view,
        }
    }

    pub fn from_tmem_data(gfx: &GfxContext, tile: &Tile, tmem_data: &[u64]) -> Self {
        let width = tile.size.width() as u32;
        let height = tile.size.height() as u32;
        let format = tile.descriptor.format;

        let tmem_line_len = ((width as usize * (4 << format.1)) + 63) / 64;
        let tmem_line_count = (height as usize).min(tmem_data.len() / tmem_line_len);

        let mut buf: [u64; 4096] = [0; 4096];

        let buf_start = deinterleave_tmem_data(
            &mut buf,
            &tmem_data[tile.descriptor.tmem_addr as usize..],
            tmem_line_len,
            tmem_line_count,
        );

        let output = decode_texture(
            &mut buf,
            buf_start,
            &tmem_data[256..],
            (tile.descriptor.palette as usize) << 4,
            format,
        );

        Texture::new(
            gfx,
            width,
            tmem_line_count as u32,
            &output[0..(width as usize * tmem_line_count * 4)],
        )
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

fn deinterleave_tmem_data(
    buf: &mut [u64],
    tmem_data: &[u64],
    tmem_line_len: usize,
    tmem_line_count: usize,
) -> usize {
    let buf_start = buf.len() - tmem_line_len * tmem_line_count;

    let mut buf_index = buf_start;
    let mut tmem_index = 0;

    for line in 0..tmem_line_count {
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

fn decode_texture<'a>(
    buf: &'a mut [u64],
    buf_start: usize,
    tlut: &[u64],
    tlut_start: usize,
    format: TextureFormat,
) -> &'a [u8] {
    let output_start = match format {
        (Format::Rgba, 3) => buf_start,
        (Format::Rgba, 2) => decode_color16(buf, buf_start, gfx::decode_rgba16),
        (Format::ClrIndex, 1) => decode_color8(buf, buf_start, |word| {
            gfx::decode_rgba16((tlut[word as usize] as u16).swap_bytes())
        }),
        (Format::ClrIndex, 0) => decode_color4(buf, buf_start, |word| {
            (
                gfx::decode_rgba16((tlut[tlut_start + (word >> 4) as usize] as u16).swap_bytes()),
                gfx::decode_rgba16((tlut[tlut_start + (word & 15) as usize] as u16).swap_bytes()),
            )
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
        (Format::I, 2) => decode_color16(buf, buf_start, |word| {
            // Unofficial
            let red = (word >> 8) as u8;
            let green = word as u8;
            let alpha = (word & 1) as u8 * 255;
            u32::from_le_bytes([red, green, red, alpha])
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
        _ => panic!("Unsupported TMEM texture format: {:?}", format),
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
