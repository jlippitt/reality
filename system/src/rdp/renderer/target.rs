use super::Rect;
use crate::gfx::GfxContext;
use crate::rdram::Rdram;
use std::mem;
use tracing::{debug, trace};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ColorImageFormat {
    Index8,
    Rgba16,
    #[default]
    Rgba32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ColorImage {
    pub dram_addr: u32,
    pub width: u32,
    pub format: ColorImageFormat,
}

pub struct TargetOutput {
    pub color_texture: wgpu::Texture,
    pub depth_texture: wgpu::Texture,
    pub sync_buffer: wgpu::Buffer,
}

pub struct Target {
    color_image: ColorImage,
    scissor: Rect,
    scissor_bind_group: wgpu::BindGroup,
    scissor_buffer: wgpu::Buffer,
    output: Option<TargetOutput>,
    dirty: bool,
    synced: bool,
}

impl Target {
    pub fn new(gfx: &GfxContext, scissor_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let scissor_buffer = gfx.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("RDP Scissor Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: mem::size_of::<[f32; 4]>() as u64,
            mapped_at_creation: false,
        });

        let scissor_bind_group = gfx.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("RDP Scissor Bind Group"),
            layout: scissor_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scissor_buffer.as_entire_binding(),
            }],
        });

        Self {
            color_image: ColorImage::default(),
            scissor: Rect::default(),
            scissor_bind_group,
            scissor_buffer,
            output: None,
            dirty: true,
            synced: false,
        }
    }

    pub fn scissor(&self) -> &Rect {
        &self.scissor
    }

    pub fn color_image(&self) -> &ColorImage {
        &self.color_image
    }

    pub fn scissor_bind_group(&self) -> &wgpu::BindGroup {
        &self.scissor_bind_group
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn request_sync(&mut self) {
        self.synced = false;
    }

    pub fn set_color_image(&mut self, color_image: ColorImage) {
        self.dirty |= color_image != self.color_image;
        self.color_image = color_image;
        trace!("  Color Image: {:?}", self.color_image);
        trace!("  Dirty: {}", self.dirty);
    }

    pub fn set_scissor(&mut self, scissor: Rect) {
        self.dirty |= scissor != self.scissor;
        self.scissor = scissor;
        trace!("  Scissor: {:?}", self.scissor);
        trace!("  Dirty: {}", self.dirty);
    }

    pub fn output(&self) -> Option<&TargetOutput> {
        self.output.as_ref()
    }

    pub fn upload_buffers(&self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.scissor_buffer,
            0,
            bytemuck::cast_slice(&[
                self.scissor.left,
                self.scissor.top,
                self.scissor.width(),
                self.scissor.height(),
            ]),
        );
    }

    pub fn update(&mut self, gfx: &GfxContext, rdram: &mut Rdram) {
        if self.output.is_some() {
            if !self.dirty {
                return;
            }

            // Make sure contents of previous image are written to RDRAM
            self.sync(gfx, rdram);
        };

        // Width must be padded to a 64-byte boundary for 'copy to buffer' to work
        // TODO: Width is exclusive in 1-Cycle/2-Cycle mode
        let width = (self.scissor.width() as u32 + 63) & !63;
        let height = self.scissor.height() as u32;

        if width == 0 || height == 0 {
            panic!("Cannot create target texture with size of zero")
        }

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let color_texture = gfx.device().create_texture(&wgpu::TextureDescriptor {
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
        });

        let depth_texture = gfx.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("RDP Target Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let sync_buffer = gfx.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("RDP Target Sync Buffer"),
            size: width as u64 * height as u64 * 4,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // TODO: Upload pixels from existing Color Image

        self.output = Some(TargetOutput {
            color_texture,
            depth_texture,
            sync_buffer,
        });

        self.dirty = false;
    }

    pub fn sync(&mut self, gfx: &GfxContext, rdram: &mut Rdram) {
        if self.synced {
            return;
        }

        if self.color_image.width == 0 {
            debug!("  Attempting to sync target output with zero-width color image");
        };

        let Some(output) = &mut self.output else {
            debug!("  Attempting to sync target output with no output texture set");
            return;
        };

        debug!("  Writing output texture to RDRAM");

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
            let mut ram_addr = self.color_image.dram_addr as usize;

            // TODO: What happens when color image width is not the same as texture width?
            match self.color_image.format {
                ColorImageFormat::Index8 => todo!("Index8 output format"),
                ColorImageFormat::Rgba16 => {
                    for _ in 0..output.color_texture.height() {
                        // TODO: Make a persistent Vec buffer for the pixel data (so we don't allocate here)
                        let pixels: Vec<u8> = pixel_data
                            [buf_addr..(buf_addr + self.color_image.width as usize * 4)]
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
                        ram_addr += self.color_image.width as usize * 2;
                    }
                }
                ColorImageFormat::Rgba32 => {
                    for _ in 0..output.color_texture.height() {
                        rdram.write_block(
                            ram_addr,
                            &pixel_data[buf_addr..(buf_addr + self.color_image.width as usize * 4)],
                        );
                        buf_addr += output.color_texture.width() as usize * 4;
                        ram_addr += self.color_image.width as usize * 4;
                    }
                }
            }
        } else {
            panic!("Failed to sync with WGPU");
        }

        output.sync_buffer.unmap();

        self.synced = true;
    }
}
