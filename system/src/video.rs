use crate::gfx::GfxContext;
use crate::interrupt::{RcpIntType, RcpInterrupt};
use crate::memory::{Size, WriteMask};
use crate::rdram::Rdram;
use crate::{RCP_CLOCK_RATE, VIDEO_DAC_RATE};
use framebuffer::Framebuffer;
use regs::Regs;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tracing::{debug, trace};
use upscaler::Upscaler;

mod framebuffer;
mod regs;
mod upscaler;

pub struct VideoInterface {
    regs: Regs,
    cycles_remaining: u32,
    cycles_per_line: u32,
    frame_counter: u64,
    rcp_int: Arc<Mutex<RcpInterrupt>>,
    upscaler: Upscaler,
    frame_buffer: Framebuffer,
}

impl VideoInterface {
    pub fn new(
        rcp_int: Arc<Mutex<RcpInterrupt>>,
        gfx: &GfxContext,
        skip_pif_rom: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let upscaler = Upscaler::new(gfx.device(), gfx.output_format());

        let frame_buffer = Framebuffer::new(gfx.device(), upscaler.texture_bind_group_layout());

        let mut regs = Regs::default();

        if skip_pif_rom {
            regs.v_intr.set_half_line(1023);
        }

        let cycles_per_line = calc_cycles_per_line(regs.h_sync.h_sync());

        Ok(Self {
            regs,
            cycles_remaining: cycles_per_line,
            cycles_per_line,
            frame_counter: 0,
            rcp_int,
            upscaler,
            frame_buffer,
        })
    }

    pub fn present(&mut self, gfx: &GfxContext) -> Result<(), wgpu::SurfaceError> {
        let output = gfx.surface_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = gfx
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.upscaler
            .render(&mut encoder, &view, self.frame_buffer.bind_group());

        gfx.queue().submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    #[inline(always)]
    pub fn step(&mut self, rdram: &Rdram, gfx: &GfxContext) -> bool {
        self.cycles_remaining -= 1;

        if self.cycles_remaining > 0 {
            return false;
        }

        self.step_inner(rdram, gfx)
    }

    fn step_inner(&mut self, rdram: &Rdram, gfx: &GfxContext) -> bool {
        self.cycles_remaining = self.cycles_per_line;

        let mut half_line = self.regs.v_current.half_line() + 2;
        let mut frame_done = false;

        if half_line > self.regs.v_sync.v_sync() {
            let serrate = self.regs.ctrl.serrate() as u32;
            half_line = (half_line & serrate) ^ serrate;
            self.frame_counter += 1;
            debug!("Frame {} (Field={})", self.frame_counter, half_line & 1);
            self.render(rdram, gfx);
            frame_done = true;
        }

        if half_line == self.regs.v_intr.half_line() {
            self.rcp_int.lock().unwrap().raise(RcpIntType::VI);
        }

        self.regs.v_current.set_half_line(half_line);
        trace!("VI_V_CURRENT: {:?}", self.regs.v_current);

        frame_done
    }

    fn render(&mut self, rdram: &Rdram, gfx: &GfxContext) {
        let video_width = self.regs.h_video.width() * self.regs.x_scale.scale() / 1024;

        let video_height = (self.regs.v_video.width() >> 1) * self.regs.y_scale.scale() / 1024;

        self.frame_buffer.resize(
            gfx.device(),
            self.upscaler.texture_bind_group_layout(),
            self.regs.ctrl.aa_mode(),
            video_width,
            video_height,
        );

        // TODO: We should technically upload each display pixel as it occurs
        // rather than doing things all at once at the end of the frame.
        self.frame_buffer.upload(
            gfx.queue(),
            rdram,
            self.regs.ctrl.display_mode(),
            self.regs.origin.origin(),
            self.regs.width.width(),
        );
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        T::truncate_u32(match address >> 2 {
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
            4 => self.rcp_int.lock().unwrap().clear(RcpIntType::VI),
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
    debug!("VI Cycles Per Line: {}", value);
    value
}
