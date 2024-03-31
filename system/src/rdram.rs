use crate::cpu::Size;
use crate::memory::WriteMask;
use regs::{RiConfig, RiMode, RiSelect};
use tracing::trace;

mod regs;

struct Interface {
    mode: RiMode,
    config: RiConfig,
    select: RiSelect,
}

pub struct Rdram {
    ri: Interface,
}

impl Rdram {
    pub fn new() -> Self {
        Self {
            ri: Interface {
                mode: RiMode::new(),
                config: RiConfig::new(),
                select: RiSelect::new(),
            },
        }
    }

    pub fn read_interface<T: Size>(&self, address: u32) -> T {
        T::from_u32(match address >> 2 {
            0 => self.ri.mode.into(),
            3 => self.ri.select.into(),
            _ => todo!("RI Register Read: {:08X}", address),
        })
    }

    pub fn write_interface<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        match address >> 2 {
            0 => {
                mask.write(&mut self.ri.mode);
                trace!("RI_MODE: {:?}", self.ri.mode);
            }
            1 => {
                mask.write(&mut self.ri.config);
                trace!("RI_CONFIG: {:?}", self.ri.config);
            }
            2 => {
                // This is a NOP as it's not real hardware...
                trace!("RI_CURRENT_LOAD complete");
            }
            3 => {
                mask.write(&mut self.ri.select);
                trace!("RI_SELECT: {:?}", self.ri.select);
                assert_eq!(0b0100, self.ri.select.rsel());
                assert_eq!(0b0001, self.ri.select.tsel());
            }
            _ => todo!("RI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }
}
