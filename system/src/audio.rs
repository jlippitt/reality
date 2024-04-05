use crate::memory::{Size, WriteMask};
use regs::Regs;
use tracing::warn;

mod regs;

pub struct AudioInterface {
    regs: Regs,
}

impl AudioInterface {
    pub fn new() -> Self {
        Self {
            regs: Regs::default(),
        }
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        todo!("AI Register Read: {:08X}", address);
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        match address >> 2 {
            0 => mask.write_reg("AI_DRAM_ADDR", &mut self.regs.dram_addr),
            1 => {
                mask.write_reg("AI_LENGTH", &mut self.regs.length);

                if self.regs.length.length() > 0 {
                    todo!("AI DMA Transfers");
                }
            }
            2 => {
                mask.write_reg("AI_CONTROL", &mut self.regs.control);

                if self.regs.control.dma_enable() {
                    warn!("TODO: AI DMA Transfers");
                }
            }
            3 => {
                // TODO: Acknowledge AI interrupt
            }
            4 => mask.write_reg("AI_DACRATE", &mut self.regs.dacrate),
            5 => mask.write_reg("AI_BITRATE", &mut self.regs.bitrate),
            _ => todo!("AI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }
}
