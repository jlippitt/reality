use crate::rdram::Rdram;
use std::error::Error;

pub struct DisplayTarget<T: wgpu::WindowHandle + 'static> {
    pub window: T,
    pub width: u32,
    pub height: u32,
}

pub struct GfxContext {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl GfxContext {
    pub fn new(
        display_target: DisplayTarget<impl wgpu::WindowHandle>,
    ) -> Result<Self, Box<dyn Error>> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(display_target.window)?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or("Failed to find adapter compatible with window surface")?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default().using_resolution(adapter.limits()),
                label: None,
            },
            None,
        ))?;

        let capabilities = surface.get_capabilities(&adapter);

        let output_format = capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: output_format,
            width: display_target.width,
            height: display_target.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        Ok(Self {
            surface,
            device,
            queue,
            config,
        })
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn surface_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    pub fn output_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == self.config.width && height == self.config.height {
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }
}

pub fn decode_rgba16(word: u16) -> u32 {
    let red = ((word >> 11) as u8 & 31) << 3;
    let green = ((word >> 6) as u8 & 31) << 3;
    let blue = ((word >> 1) as u8 & 31) << 3;
    let alpha = (word as u8 & 1) * 255;
    u32::from_le_bytes([red, green, blue, alpha])
}

pub fn copy_image_rgba16(
    rdram: &Rdram,
    pixel_buf: &mut [u8],
    dram_addr: u32,
    dram_width: u32,
    image_width: u32,
    image_height: u32,
) {
    let src_pitch = dram_width as usize * 2;
    let dst_pitch = image_width as usize * 4;
    let dst_display = dst_pitch.min(dram_width as usize * 4);

    let mut src = dram_addr as usize;
    let mut dst = 0;

    for _ in 0..image_height {
        let draw_area: &mut [u16] =
            bytemuck::cast_slice_mut(&mut pixel_buf[dst..(dst + dst_display)]);

        let read_start = draw_area.len() / 2;

        rdram.read_block(src, &mut draw_area[read_start..]);

        for index in 0..(draw_area.len() / 2) {
            let word = draw_area[read_start + index].swap_bytes();
            let color = decode_rgba16(word);
            bytemuck::cast_slice_mut::<u16, u32>(draw_area)[index] = color;
        }

        pixel_buf[(dst + dst_display)..(dst + dst_pitch)].fill(0);

        src += src_pitch;
        dst += dst_pitch;
    }
}

pub fn copy_image_rgba32(
    rdram: &Rdram,
    pixel_buf: &mut [u8],
    dram_addr: u32,
    dram_width: u32,
    image_width: u32,
    image_height: u32,
) {
    let src_pitch = dram_width as usize * 4;
    let dst_pitch = image_width as usize * 4;
    let dst_display = dst_pitch.min(src_pitch);

    let mut src = dram_addr as usize;
    let mut dst = 0;

    for _ in 0..image_height {
        let draw_area: &mut [u32] =
            bytemuck::cast_slice_mut(&mut pixel_buf[dst..(dst + dst_display)]);

        rdram.read_block(src, draw_area);

        pixel_buf[(dst + dst_display)..(dst + dst_pitch)].fill(0);

        src += src_pitch;
        dst += dst_pitch;
    }
}
