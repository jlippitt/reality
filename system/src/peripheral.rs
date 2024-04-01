use crate::cpu::Size;
use crate::memory::{Memory, WriteMask};
use crate::rdram::Rdram;
use tracing::{trace, warn};

struct Dma {
    len: u32,
    write: bool,
}

pub struct PeripheralInterface {
    dram_addr: u32,
    cart_addr: u32,
    dma: Option<Dma>,
}

impl PeripheralInterface {
    pub fn new() -> Self {
        Self {
            dram_addr: 0,
            cart_addr: 0,
            dma: None,
        }
    }

    pub fn step(&mut self, rdram: &mut Rdram, rom: &mut Memory) {
        if let Some(dma) = &mut self.dma {
            let block_address = self.dram_addr;
            let block_len = dma.len.min(128);

            assert!((block_address & 3) == 0);
            assert!((block_len & 3) == 0);

            let mut buf = [0u32; 32];
            let data = &mut buf[0..((block_len >> 2) as usize)];

            if dma.write {
                rom.read_block(self.cart_addr & 0x00ff_ffff, data);
                rdram.write_block(self.dram_addr & 0x00ff_ffff, data);

                trace!(
                    "PI DMA: {} bytes written from {:08X} to {:08X}",
                    block_len,
                    self.cart_addr,
                    self.dram_addr,
                );
            } else {
                rdram.read_block(self.dram_addr, data);
                rom.write_block(self.cart_addr, data);

                trace!(
                    "PI DMA: {} bytes read from {:08X} to {:08X}",
                    block_len,
                    self.cart_addr,
                    self.dram_addr,
                );
            }

            // TODO: Can these wrap?
            self.dram_addr += block_len;
            self.cart_addr += block_len;
            dma.len -= block_len;

            if dma.len == 0 {
                self.dma = None;
            }
        }
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        T::from_u32(match address >> 2 {
            4 => {
                // TODO: Interrupt status
                let mut value: u32 = 0;

                if self.dma.is_some() {
                    value |= 0x01;
                }

                value
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
            2 => {
                self.dma = Some(Dma {
                    len: (mask.raw() & 0x00ff_ffff) + 1,
                    write: false,
                })
            }
            3 => {
                self.dma = Some(Dma {
                    len: (mask.raw() & 0x00ff_ffff) + 1,
                    write: true,
                })
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
