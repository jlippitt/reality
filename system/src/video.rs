use super::memory::WriteMask;
use crate::cpu::Size;
use regs::{HVideo, VIntr};
use tracing::{trace, warn};

mod regs;

pub struct VideoInterface {
    v_intr: VIntr,
    h_video: HVideo,
}

impl VideoInterface {
    pub fn new() -> Self {
        Self {
            v_intr: VIntr::new(),
            h_video: HVideo::new(),
        }
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        todo!("VI Register Read: {:08X}", address);
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        match address >> 2 {
            3 => {
                mask.write(&mut self.v_intr);
                trace!("VI_V_INTR: {:?}", self.v_intr);
            }
            4 => warn!("TODO: Acknowledge VI interrupt"),
            9 => {
                mask.write(&mut self.h_video);
                trace!("VI_H_VIDEO: {:?}", self.h_video);
            }
            _ => todo!("VI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }
}