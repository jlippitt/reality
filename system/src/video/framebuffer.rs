use super::regs::{AntiAliasMode, DisplayMode};
use crate::gfx;
use crate::rdram::Rdram;
use std::sync::RwLock;

pub struct Framebuffer {
    sampler_linear: wgpu::Sampler,
    sampler_nearest: wgpu::Sampler,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    aa_mode: AntiAliasMode,
    pixel_buf: Vec<u8>,
}

impl Framebuffer {
    pub fn new(device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let sampler_linear = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Framebuffer Sampler (Linear)"),
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let sampler_nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Framebuffer Sampler (Nearest)"),
            mag_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture = create_texture(device, 1, 1);
        let bind_group = create_bind_group(device, bind_group_layout, &texture, &sampler_linear);

        Self {
            sampler_linear,
            sampler_nearest,
            texture,
            bind_group,
            aa_mode: AntiAliasMode::default(),
            pixel_buf: vec![0; 4],
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        aa_mode: AntiAliasMode,
        width: u32,
        height: u32,
    ) {
        let width = width.max(1);
        let height = height.max(1);

        if width != self.texture.width() && height != self.texture.height() {
            self.texture = create_texture(device, width, height);
            self.pixel_buf
                .resize(width as usize * height as usize * 4, 0);
        } else if aa_mode == self.aa_mode {
            return;
        }

        let sampler = if aa_mode != AntiAliasMode::Off {
            &self.sampler_linear
        } else {
            &self.sampler_nearest
        };

        self.bind_group = create_bind_group(device, bind_group_layout, &self.texture, sampler);
    }

    pub fn upload(
        &mut self,
        queue: &wgpu::Queue,
        rdram: &RwLock<Rdram>,
        display_mode: DisplayMode,
        origin: u32,
        buffer_width: u32,
    ) {
        let video_width = self.texture.width();
        let video_height = self.texture.height();

        match display_mode {
            DisplayMode::Blank => self.pixel_buf.fill(0),
            DisplayMode::Reserved => panic!("Use of reserved display mode"),
            DisplayMode::Color16 => gfx::copy_image_rgba16(
                &rdram.read().unwrap(),
                &mut self.pixel_buf,
                origin,
                buffer_width,
                video_width,
                video_height,
            ),
            DisplayMode::Color32 => gfx::copy_image_rgba32(
                &rdram.read().unwrap(),
                &mut self.pixel_buf,
                origin,
                buffer_width,
                video_width,
                video_height,
            ),
        }

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.pixel_buf,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(video_width * 4),
                rows_per_image: Some(video_height),
            },
            self.texture.size(),
        )
    }
}

fn create_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Framebuffer Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn create_bind_group(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    texture: &wgpu::Texture,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Framebuffer Texture Bind Group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}
