use super::{Format, Rect, TextureFormat};
use crate::gfx::{self, GfxContext};
use crate::rdram::Rdram;
use fill_color::FillColor;
use std::mem;
use tracing::warn;
use tracing::{debug, trace};
use wgpu::util::DeviceExt;

mod fill_color;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ColorImage {
    pub dram_addr: u32,
    pub width: u32,
    pub format: TextureFormat,
}

pub struct TargetOutput {
    pub color_image: ColorImage,
    pub color_texture: wgpu::Texture,
    pub depth_texture: wgpu::Texture,
    pub sync_buffer: wgpu::Buffer,
}

pub struct Target {
    color_image: ColorImage,
    scissor: Rect,
    max_scissor_height: u32,
    image_size_buffer: wgpu::Buffer,
    image_size_bind_group_layout: wgpu::BindGroupLayout,
    image_size_bind_group: wgpu::BindGroup,
    fill_color: FillColor,
    output: Option<TargetOutput>,
    pixel_buf: Vec<u8>,
    synced: bool,
}

impl Target {
    pub fn new(gfx: &GfxContext) -> Self {
        let image_size_bind_group_layout =
            gfx.device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("RDP Image Size Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let image_size_buffer = gfx.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("RDP Image Size Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: mem::size_of::<[f32; 2]>() as u64,
            mapped_at_creation: false,
        });

        let image_size_bind_group = gfx.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("RDP Image Size Bind Group"),
            layout: &image_size_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: image_size_buffer.as_entire_binding(),
            }],
        });

        Self {
            color_image: ColorImage::default(),
            scissor: Rect::default(),
            max_scissor_height: 0,
            image_size_buffer,
            image_size_bind_group_layout,
            image_size_bind_group,
            fill_color: FillColor::new(gfx),
            output: None,
            pixel_buf: vec![],
            synced: false,
        }
    }

    pub fn color_image(&self) -> &ColorImage {
        &self.color_image
    }

    pub fn scissor(&self) -> &Rect {
        &self.scissor
    }

    pub fn image_size_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.image_size_bind_group_layout
    }

    pub fn image_size_bind_group(&self) -> &wgpu::BindGroup {
        &self.image_size_bind_group
    }

    pub fn fill_color(&self) -> u32 {
        self.fill_color.value()
    }

    pub fn fill_color_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        self.fill_color.bind_group_layout()
    }

    pub fn fill_color_bind_group(&self) -> &wgpu::BindGroup {
        self.fill_color.bind_group()
    }

    pub fn request_sync(&mut self) {
        self.synced = false;
    }

    pub fn set_color_image(&mut self, color_image: ColorImage) {
        self.color_image = color_image;
        trace!("  Color Image: {:?}", self.color_image);
    }

    pub fn set_scissor(&mut self, mut scissor: Rect) {
        // Zero-size scissor causes bad things to happen
        scissor.right = scissor.right.max(scissor.left + 1.0);
        scissor.bottom = scissor.bottom.max(scissor.top + 1.0);
        self.scissor = scissor;
        self.max_scissor_height = self.max_scissor_height.max(self.scissor.bottom as u32);
        trace!("  Scissor: {:?}", self.scissor);
        trace!("  Max Scissor Height: {:?}", self.max_scissor_height);
    }

    pub fn set_fill_color(&mut self, queue: &wgpu::Queue, value: u32) {
        self.fill_color.set_value(value);
        self.fill_color.upload(queue, self.color_image.format.1);
    }

    pub fn output(&self) -> Option<&TargetOutput> {
        self.output.as_ref()
    }

    pub fn update(&mut self, gfx: &GfxContext, rdram: &mut Rdram) -> bool {
        let height = self.max_scissor_height.max(self.color_image.width * 3 / 4);

        if let Some(output) = &self.output {
            if output.color_image == self.color_image && output.color_texture.height() == height {
                return true;
            };

            self.sync(gfx, rdram);
        }

        debug!(
            "  Creating new target texture: {}x{}",
            self.color_image.width, height
        );

        // Width must be padded to a 64-byte boundary for 'copy to buffer' to work
        // TODO: Width is exclusive in 1-Cycle/2-Cycle mode
        let width = (self.color_image.width + 63) & !63;

        if width == 0 || height == 0 {
            warn!("Cannot create target texture with size of zero");
            return false;
        }

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        self.pixel_buf.resize((width * height * 4) as usize, 0);

        match self.color_image.format {
            (Format::Rgba, 3) => gfx::copy_image_rgba32(
                rdram,
                &mut self.pixel_buf,
                self.color_image.dram_addr,
                self.color_image.width,
                width,
                height,
            ),
            (Format::Rgba, 2) => gfx::copy_image_rgba16(
                rdram,
                &mut self.pixel_buf,
                self.color_image.dram_addr,
                self.color_image.width,
                width,
                height,
            ),
            (Format::ClrIndex, 1) => todo!("Index8 output format"),
            _ => panic!("Unsupported Color Image format"),
        };

        let color_texture = gfx.device().create_texture_with_data(
            gfx.queue(),
            &wgpu::TextureDescriptor {
                label: Some("RDP Target Color Texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &self.pixel_buf,
        );

        bytemuck::cast_slice_mut::<u8, u16>(&mut self.pixel_buf).fill(u16::MAX);

        let depth_texture = gfx.device().create_texture_with_data(
            gfx.queue(),
            &wgpu::TextureDescriptor {
                label: Some("RDP Target Depth Texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth16Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &self.pixel_buf,
        );

        gfx.queue().write_texture(
            wgpu::ImageCopyTexture {
                texture: &depth_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::DepthOnly,
            },
            &self.pixel_buf,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            size,
        );

        let sync_buffer = gfx.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("RDP Target Sync Buffer"),
            size: width as u64 * height as u64 * 4,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Image dimensions have changed, so update image size
        gfx.queue().write_buffer(
            &self.image_size_buffer,
            0,
            bytemuck::cast_slice(&[width as f32, height as f32]),
        );

        // BPP may have changed, so update fill color
        self.fill_color
            .upload(gfx.queue(), self.color_image.format.1);

        self.output = Some(TargetOutput {
            color_image: self.color_image.clone(),
            color_texture,
            depth_texture,
            sync_buffer,
        });

        true
    }

    pub fn sync(&mut self, gfx: &GfxContext, rdram: &mut Rdram) {
        if self.synced {
            return;
        }

        let Some(output) = &mut self.output else {
            trace!("  Attempting to sync target output with no output texture set");
            return;
        };

        self.max_scissor_height = self.max_scissor_height.max(self.scissor.height() as u32);

        debug!(
            "  Writing output texture to RDRAM {:08X}: {}x{}",
            output.color_image.dram_addr, output.color_image.width, self.max_scissor_height,
        );

        let mut encoder = gfx
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("RDP Target Sync Command Encoder"),
            });

        // Copy the color texture into RDRAM
        encoder.copy_texture_to_buffer(
            output.color_texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer: &output.sync_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(output.color_texture.width() * 4),
                    rows_per_image: Some(output.color_texture.height()),
                },
            },
            output.color_texture.size(),
        );

        gfx.queue().submit(std::iter::once(encoder.finish()));

        // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();

        let buffer_slice = output.sync_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        gfx.device().poll(wgpu::Maintain::Wait);

        if let Some(Ok(())) = pollster::block_on(receiver.receive()) {
            let pixel_data = &output.sync_buffer.slice(..).get_mapped_range();

            let mut buf_addr = 0;
            let mut ram_addr = output.color_image.dram_addr as usize;

            // TODO: What happens when color image width is not the same as texture width?
            match output.color_image.format {
                (Format::Rgba, 3) => {
                    for _ in 0..self.max_scissor_height {
                        rdram.write_block(
                            ram_addr,
                            &pixel_data
                                [buf_addr..(buf_addr + output.color_texture.width() as usize * 4)],
                        );
                        buf_addr += output.color_texture.width() as usize * 4;
                        ram_addr += output.color_image.width as usize * 4;
                    }
                }
                (Format::Rgba, 2) => {
                    for _ in 0..self.max_scissor_height {
                        // TODO: Make a persistent Vec buffer for the pixel data (so we don't allocate here)
                        let pixels: Vec<u8> = pixel_data
                            [buf_addr..(buf_addr + output.color_texture.width() as usize * 4)]
                            .chunks_exact(4)
                            .flat_map(|chunk| {
                                let color = ((chunk[0] as u16 >> 3) << 11)
                                    | ((chunk[1] as u16 >> 3) << 6)
                                    | ((chunk[2] as u16 >> 3) << 1)
                                    | (chunk[3] as u16 >> 7);

                                color.to_be_bytes()
                            })
                            .collect();

                        rdram.write_block(ram_addr, &pixels);
                        buf_addr += output.color_texture.width() as usize * 4;
                        ram_addr += output.color_image.width as usize * 2;
                    }
                }
                (Format::ClrIndex, 1) => todo!("Index8 output format"),
                _ => panic!("Unsupported Color Image format"),
            }
        } else {
            panic!("Failed to sync with WGPU");
        }

        output.sync_buffer.unmap();

        self.max_scissor_height = 0;
        trace!("  Max Scissor Height: {:?}", self.max_scissor_height);

        self.synced = true;
    }
}
