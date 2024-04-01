use crate::cpu::Size;
use crate::memory::WriteMask;
use tracing::{trace, warn};

pub struct PeripheralInterface {
    dram_addr: u32,
    cart_addr: u32,
}

impl PeripheralInterface {
    pub fn new() -> Self {
        Self {
            dram_addr: 0,
            cart_addr: 0,
        }
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        T::from_u32(match address >> 2 {
            4 => {
                // TODO: DMA status
                0
            }
            _ => todo!("PI Register Read: {:08X}", address),
        })
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        match address >> 2 {
            0 => {
                mask.write(&mut self.dram_addr);
                trace!("PI_DRAM_ADDR: {:08X}", self.dram_addr);
            }
            1 => {
                mask.write(&mut self.cart_addr);
                trace!("PI_CART_ADDR: {:08X}", self.cart_addr);
            }
            4 => {
                let raw = mask.raw();

                if (raw & 0x01) != 0 {
                    warn!("TODO: Reset PI DMA controller");
                }

                if (raw & 0x02) != 0 {
                    warn!("TODO: Acknowledge PI interrupt");
                }
            }
            5..=12 => warn!("TODO: PI DOM registers"),
            _ => todo!("PI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }
}
