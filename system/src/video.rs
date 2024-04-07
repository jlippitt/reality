use super::interrupt::{RcpIntType, RcpInterrupt};
use super::memory::{Size, WriteMask};
use super::{RCP_CLOCK_RATE, VIDEO_DAC_RATE};
use crate::rdram::Rdram;
use framebuffer::Framebuffer;
use regs::Regs;
use std::error::Error;
use tracing::{debug, trace};
use upscaler::Upscaler;

mod framebuffer;
mod regs;
mod upscaler;

pub struct DisplayTarget<T: wgpu::WindowHandle + 'static> {
    pub window: T,
    pub width: u32,
    pub height: u32,
}

pub struct VideoInterface {
    regs: Regs,
    current_cycles: u32,
    cycles_per_line: u32,
    frame_counter: u64,
    rcp_int: RcpInterrupt,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    upscaler: Upscaler,
    frame_buffer: Framebuffer,
}

impl VideoInterface {
    pub fn new(
        rcp_int: RcpInterrupt,
        display_target: DisplayTarget<impl wgpu::WindowHandle>,
        skip_pif_rom: bool,
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
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let upscaler = Upscaler::new(&device, output_format);

        let frame_buffer = Framebuffer::new(&device, upscaler.texture_bind_group_layout());

        let mut regs = Regs::default();

        if skip_pif_rom {
            regs.v_intr.set_half_line(1023);
        }

        let cycles_per_line = calc_cycles_per_line(regs.h_sync.h_sync());

        Ok(Self {
            regs,
            current_cycles: 0,
            cycles_per_line,
            frame_counter: 0,
            rcp_int,
            surface,
            device,
            queue,
            config,
            upscaler,
            frame_buffer,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == self.config.width && height == self.config.height {
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self, rdram: &Rdram) -> Result<(), wgpu::SurfaceError> {
        let video_width = self.regs.h_video.width() * self.regs.x_scale.scale() / 1024;

        let video_height = (self.regs.v_video.width() >> 1) * self.regs.y_scale.scale() / 1024;

        self.frame_buffer.resize(
            &self.device,
            self.upscaler.texture_bind_group_layout(),
            self.regs.ctrl.aa_mode(),
            video_width,
            video_height,
        );

        // TODO: We should technically upload each display pixel as it occurs
        // rather than doing things all at once at the end of the frame.
        self.frame_buffer.upload(
            &self.queue,
            rdram,
            self.regs.ctrl.display_mode(),
            self.regs.origin.origin(),
            self.regs.width.width(),
        );

        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.upscaler
            .render(&mut encoder, &view, self.frame_buffer.bind_group());

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn step(&mut self) -> bool {
        let mut frame_done = false;

        self.current_cycles += 1;

        if self.current_cycles >= self.cycles_per_line {
            self.current_cycles -= self.cycles_per_line;

            let mut half_line = self.regs.v_current.half_line() + 2;

            if half_line > self.regs.v_sync.v_sync() {
                let serrate = self.regs.ctrl.serrate() as u32;
                half_line = (half_line & serrate) ^ serrate;
                self.frame_counter += 1;
                debug!("Frame {} (Field={})", self.frame_counter, half_line & 1);
                frame_done = true;
            }

            if half_line == self.regs.v_intr.half_line() {
                self.rcp_int.raise(RcpIntType::VI);
            }

            self.regs.v_current.set_half_line(half_line);
            trace!("VI_V_CURRENT: {:?}", self.regs.v_current);
        }

        frame_done
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        T::from_u32(match address >> 2 {
            0 => self.regs.ctrl.into(),
            1 => self.regs.origin.into(),
            2 => self.regs.width.into(),
            3 => self.regs.v_intr.into(),
            4 => self.regs.v_current.into(),
            5 => self.regs.burst.into(),
            6 => self.regs.v_sync.into(),
            7 => self.regs.h_sync.into(),
            8 => self.regs.h_sync_leap.into(),
            9 => self.regs.h_video.into(),
            10 => self.regs.v_video.into(),
            11 => self.regs.v_burst.into(),
            12 => self.regs.x_scale.into(),
            13 => self.regs.y_scale.into(),
            14 => self.regs.test_addr.into(),
            15 => todo!("VI_STAGED_DATA read"),
            _ => unimplemented!("VI Register Read: {:08X}", address),
        })
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        match address >> 2 {
            0 => mask.write_reg("VI_CTRL", &mut self.regs.ctrl),
            1 => mask.write_reg_hex("VI_ORIGIN", &mut self.regs.origin),
            2 => mask.write_reg("VI_WIDTH", &mut self.regs.width),
            3 => mask.write_reg("VI_V_INTR", &mut self.regs.v_intr),
            4 => self.rcp_int.clear(RcpIntType::VI),
            5 => mask.write_reg("VI_BURST", &mut self.regs.burst),
            6 => mask.write_reg("VI_V_SYNC", &mut self.regs.v_sync),
            7 => {
                mask.write_reg("VI_H_SYNC", &mut self.regs.h_sync);
                self.cycles_per_line = calc_cycles_per_line(self.regs.h_sync.h_sync());
            }
            8 => mask.write_reg("VI_H_SYNC_LEAP", &mut self.regs.h_sync_leap),
            9 => mask.write_reg("VI_H_VIDEO", &mut self.regs.h_video),
            10 => mask.write_reg("VI_V_VIDEO", &mut self.regs.v_video),
            11 => mask.write_reg("VI_V_BURST", &mut self.regs.v_burst),
            12 => mask.write_reg("VI_X_SCALE", &mut self.regs.x_scale),
            13 => mask.write_reg("VI_Y_SCALE", &mut self.regs.y_scale),
            14 => mask.write_reg("VI_TEST_ADDR", &mut self.regs.test_addr),
            15 => mask.write_reg("VI_STAGED_DATA", &mut self.regs.staged_data),
            _ => unimplemented!("VI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }
}

fn calc_cycles_per_line(h_sync: u32) -> u32 {
    let value = (RCP_CLOCK_RATE * (h_sync + 1) as f64 / VIDEO_DAC_RATE) as u32;
    debug!("Cycles Per Line: {}", value);
    value
}
