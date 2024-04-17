use super::Rect;
use crate::gfx::GfxContext;
use crate::rdram::Rdram;
use tracing::debug;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ColorImageFormat {
    Index8,
    Rgba16,
    Rgba32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ColorImage {
    dram_addr: u32,
    width: u32,
    format: ColorImageFormat,
}

pub struct TargetOutput {
    pub color_texture: wgpu::Texture,
    pub sync_buffer: wgpu::Buffer,
}

pub struct Target {
    color_image: Option<ColorImage>,
    scissor: Option<Rect>,
    output: Option<TargetOutput>,
    dirty: bool,
    synced: bool,
}

impl Target {
    pub fn new() -> Self {
        Self {
            color_image: None,
            scissor: None,
            output: None,
            dirty: true,
            synced: false,
        }
    }

    pub fn set_color_image(&mut self, dram_addr: u32, width: u32, format: ColorImageFormat) {
        let new_image = ColorImage {
            dram_addr,
            width,
            format,
        };

        self.dirty |= !self
            .color_image
            .as_ref()
            .is_some_and(|image| *image != new_image);

        self.color_image = Some(new_image);
    }

    pub fn set_scissor(&mut self, rect: Rect) {
        self.dirty |= !self
            .scissor
            .as_ref()
            .is_some_and(|scissor| *scissor == rect);

        self.scissor = Some(rect);
    }

    pub fn output(&self) -> Option<&TargetOutput> {
        self.output.as_ref()
    }

    pub fn update(&mut self, gfx: &GfxContext, rdram: &mut Rdram) {
        if self.output.is_some() {
            if !self.dirty {
                return;
            }

            // Make sure contents of previous image are written to RDRAM
            self.sync(gfx, rdram);
        };

        let Some(_color_image) = &self.color_image else {
            debug!("Attempting to update target output with no Color Image set");
            return;
        };

        let Some(scissor) = &self.scissor else {
            debug!("Attempting to update target output with no Scissor Rect set");
            return;
        };

        // Width must be padded to a 64-byte boundary for 'copy to buffer' to work
        // TODO: Width is exclusive in 1-Cycle/2-Cycle mode
        let width = (scissor.width() + 63) & !63;
        let height = scissor.height();

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

        let sync_buffer = gfx.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("RDP Target Sync Buffer"),
            size: width as u64 * height as u64 * 4,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // TODO: Upload pixels from existing Color Image

        self.output = Some(TargetOutput {
            color_texture,
            sync_buffer,
        });
    }

    pub fn sync(&mut self, _gfx: &GfxContext, _rdram: &mut Rdram) {
        if self.synced {
            return;
        }

        let Some(_color_image) = &self.color_image else {
            debug!("Attempting to sync target output with no Color Image set");
            return;
        };

        let Some(_scissor) = &self.scissor else {
            debug!("Attempting to sync target output with no Scissor Rect set");
            return;
        };

        let Some(_output) = &mut self.output else {
            debug!("Attempting to sync target output with no output texture set");
            return;
        };

        debug!("Writing output texture to RDRAM");

        // TODO: Write pixels to existing Color Image

        self.synced = true;
    }
}
