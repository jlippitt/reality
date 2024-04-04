use crate::memory::{Size, WriteMask};
use regs::{DramAddr, Status};
use tracing::debug;

mod regs;

pub struct SerialInterface {
    dram_addr: DramAddr,
    status: Status,
}

impl SerialInterface {
    pub fn new() -> Self {
        Self {
            dram_addr: DramAddr::new(),
            status: Status::new(),
        }
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        T::from_u32(match address >> 2 {
            6 => self.status.into(),
            _ => todo!("SI Register Read: {:08X}", address),
        })
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        match address >> 2 {
            0 => {
                mask.write(&mut self.dram_addr);
                debug!("SI_DRAM_ADDR: {:?}", self.dram_addr);
            }
            6 => {
                // TODO: Acknowledge SI interrupt
            }
            _ => todo!("SI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }
}
