use super::memory::{Size, WriteMask};
use regs::{Burst, Ctrl, HSync, HSyncLeap, HVideo, Origin, VIntr, VSync, Width};
use tracing::{trace, warn};

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
            0 => {
                mask.write(&mut self.ctrl);
                trace!("VI_CTRL: {:?}", self.ctrl);
            }
            1 => {
                mask.write(&mut self.origin);
                trace!("VI_ORIGIN: {:08X?}", self.origin);
            }
            2 => {
                mask.write(&mut self.width);
                trace!("VI_WIDTH: {:?}", self.width);
            }
            3 => {
                mask.write(&mut self.v_intr);
                trace!("VI_V_INTR: {:?}", self.v_intr);
            }
            4 => warn!("TODO: Acknowledge VI interrupt"),
            5 => {
                mask.write(&mut self.burst);
                trace!("VI_BURST: {:?}", self.burst);
            }
            6 => {
                mask.write(&mut self.v_sync);
                trace!("VI_V_SYNC: {:?}", self.v_sync);
            }
            7 => {
                mask.write(&mut self.h_sync);
                trace!("VI_H_SYNC: {:?}", self.h_sync);
            }
            8 => {
                mask.write(&mut self.h_sync_leap);
                trace!("VI_H_SYNC_LEAP: {:?}", self.h_sync_leap);
            }
            9 => {
                mask.write(&mut self.h_video);
                trace!("VI_H_VIDEO: {:?}", self.h_video);
            }
            _ => todo!("VI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }
}
