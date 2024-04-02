use crate::memory::{Size, WriteMask};
use regs::{DramAddr, Length};
use tracing::trace;

mod regs;

pub struct AudioInterface {
    dram_addr: DramAddr,
    length: Length,
}

impl AudioInterface {
    pub fn new() -> Self {
        Self {
            dram_addr: DramAddr::new(),
            length: Length::new(),
        }
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        todo!("AI Register Read: {:08X}", address);
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        match address >> 2 {
            0 => {
                mask.write(&mut self.dram_addr);
                trace!("AI_DRAM_ADDR: {:?}", self.dram_addr);
            }
            1 => {
                mask.write(&mut self.length);
                trace!("AI_LENGTH: {:?}", self.length);

                if self.length.length() > 0 {
                    todo!("AI DMA Transfers");
                }
            }
            3 => {
                // TODO: Acknowledge AI interrupt
            }
            _ => todo!("AI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }
}
