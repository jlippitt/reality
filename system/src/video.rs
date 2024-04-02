use super::memory::{Size, WriteMask};
use regs::{Burst, Ctrl, HSync, HSyncLeap, HVideo, Origin, VIntr, VSync, Width};
use tracing::warn;

mod regs;

pub struct VideoInterface {
    ctrl: Ctrl,
    origin: Origin,
    width: Width,
    v_intr: VIntr,
    burst: Burst,
    v_sync: VSync,
    h_sync: HSync,
    h_sync_leap: HSyncLeap,
    h_video: HVideo,
}

impl VideoInterface {
    pub fn new() -> Self {
        Self {
            ctrl: Ctrl::new(),
            origin: Origin::new(),
            width: Width::new(),
            v_intr: VIntr::new(),
            burst: Burst::new(),
            v_sync: VSync::new(),
            h_sync: HSync::new(),
            h_sync_leap: HSyncLeap::new(),
            h_video: HVideo::new(),
        }
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        todo!("VI Register Read: {:08X}", address);
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        match address >> 2 {
            0 => mask.write_reg("VI_CTRL", &mut self.ctrl),
            1 => mask.write_reg_hex("VI_ORIGIN", &mut self.origin),
            2 => mask.write_reg("VI_WIDTH", &mut self.width),
            3 => mask.write_reg("VI_V_INTR", &mut self.v_intr),
            4 => warn!("TODO: Acknowledge VI interrupt"),
            5 => mask.write_reg("VI_BURST", &mut self.burst),
            6 => mask.write_reg("VI_V_SYNC", &mut self.v_sync),
            7 => mask.write_reg("VI_H_SYNC", &mut self.h_sync),
            8 => mask.write_reg("VI_H_SYNC_LEAP", &mut self.h_sync_leap),
            9 => mask.write_reg("VI_H_VIDEO", &mut self.h_video),
            _ => todo!("VI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }
}
