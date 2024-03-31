use crate::cpu::Size;
use crate::memory::WriteMask;

pub struct PeripheralInterface {
    //
}

impl PeripheralInterface {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        todo!("PI Register Write: {:08X}", address);
    }

    pub fn write<T: Size>(&self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        match address >> 2 {
            4 => {
                let raw = mask.raw();

                if (raw & 0x01) != 0 {
                    // TODO: Reset DMA controller
                }

                if (raw & 0x02) != 0 {
                    // TODO: Clear interrupt
                }
            }
            _ => todo!("PI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }
}
