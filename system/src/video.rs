use super::memory::{Size, WriteMask};
use regs::Regs;
use std::error::Error;
use tracing::warn;

mod regs;

pub struct DisplayTarget<T: wgpu::WindowHandle + 'static> {
    pub window: T,
    pub width: u32,
    pub height: u32,
}

pub struct VideoInterface {
    regs: Regs,
    h_counter: u32,
    v_counter: u32,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    _queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl VideoInterface {
    pub fn new(
        display_target: DisplayTarget<impl wgpu::WindowHandle>,
    ) -> Result<Self, Box<dyn Error>> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(display_target.window)?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or("Failed to find adapter compatible with window surface")?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
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
            regs: Regs::default(),
            v_counter: 0,
            h_counter: 0,
            surface,
            device,
            _queue: queue,
            config,
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

    pub fn step(&mut self) -> bool {
        let mut frame_done = false;
        let h_sync = self.regs.h_sync.h_sync();

        self.h_counter += 1;

        if self.h_counter >= h_sync {
            self.v_counter += 1;
            self.h_counter -= h_sync;

            if self.v_counter >= self.regs.v_sync.v_sync() {
                self.v_counter = 0;
                frame_done = true;
            }

            // TODO: Set V_CURRENT
            // TODO: VI interrupt
        }

        frame_done
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        todo!("VI Register Read: {:08X}", address);
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        match address >> 2 {
            0 => mask.write_reg("VI_CTRL", &mut self.regs.ctrl),
            1 => mask.write_reg_hex("VI_ORIGIN", &mut self.regs.origin),
            2 => mask.write_reg("VI_WIDTH", &mut self.regs.width),
            3 => mask.write_reg("VI_V_INTR", &mut self.regs.v_intr),
            4 => warn!("TODO: Acknowledge VI interrupt"),
            5 => mask.write_reg("VI_BURST", &mut self.regs.burst),
            6 => mask.write_reg("VI_V_SYNC", &mut self.regs.v_sync),
            7 => mask.write_reg("VI_H_SYNC", &mut self.regs.h_sync),
            8 => mask.write_reg("VI_H_SYNC_LEAP", &mut self.regs.h_sync_leap),
            9 => mask.write_reg("VI_H_VIDEO", &mut self.regs.h_video),
            10 => mask.write_reg("VI_V_VIDEO", &mut self.regs.v_video),
            11 => mask.write_reg("VI_V_BURST", &mut self.regs.v_burst),
            12 => mask.write_reg("VI_X_SCALE", &mut self.regs.x_scale),
            13 => mask.write_reg("VI_Y_SCALE", &mut self.regs.y_scale),
            14 => mask.write_reg("VI_TEST_ADDR", &mut self.regs.test_addr),
            15 => mask.write_reg("VI_STAGED_DATA", &mut self.regs.staged_data),
            _ => todo!("VI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }
}
