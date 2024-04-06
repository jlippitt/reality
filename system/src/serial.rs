use crate::interrupt::{RcpIntType, RcpInterrupt};
use crate::memory::{Size, WriteMask};
use crate::rdram::Rdram;
use joybus::Joybus;
use pif::Pif;
use regs::Regs;
use tracing::debug;

mod joybus;
mod pif;
mod regs;

struct Dma {
    pif_addr: u32,
    write: bool,
}

pub struct SerialInterface {
    regs: Regs,
    joybus: Joybus,
    pif: Pif,
    dma: Option<Dma>,
    rcp_int: RcpInterrupt,
}

impl SerialInterface {
    pub fn new(rcp_int: RcpInterrupt, pif_data: Option<Vec<u8>>) -> Self {
        Self {
            regs: Regs::default(),
            joybus: Joybus::new(),
            pif: Pif::new(pif_data),
            dma: None,
            rcp_int,
        }
    }

    pub fn step(&mut self, rdram: &mut Rdram) {
        if let Some(dma) = &self.dma {
            let dram_addr = self.regs.dram_addr.dram_addr();
            let mut buf = [0u32; 16];

            if dma.write {
                rdram.read_block(dram_addr, &mut buf);

                let mut pif_addr = dma.pif_addr;

                for word in buf {
                    self.pif.write(&mut self.joybus, pif_addr, word);
                    pif_addr += 4;
                }

                debug!(
                    "SI DMA: {} bytes written from {:08X} to {:04X}",
                    buf.len() * 4,
                    dram_addr,
                    dma.pif_addr,
                );
            } else {
                let mut pif_addr = dma.pif_addr;

                for word in &mut buf {
                    *word = self.pif.read(pif_addr);
                    pif_addr += 4;
                }

                rdram.write_block(dram_addr, &buf);

                debug!(
                    "SI DMA: {} bytes read from {:04X} to {:08X}",
                    buf.len() * 4,
                    dma.pif_addr,
                    dram_addr
                );
            }

            self.dma = None;
            self.rcp_int.raise(RcpIntType::SI);
        }
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        T::from_u32(match address >> 2 {
            6 => self
                .regs
                .status
                .with_interrupt(self.rcp_int.has(RcpIntType::SI))
                .into(),
            _ => todo!("SI Register Read: {:08X}", address),
        })
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        match address >> 2 {
            0 => mask.write_reg_hex("SI_DRAM_ADDR", &mut self.regs.dram_addr),
            1 => {
                self.dma = Some(Dma {
                    pif_addr: mask.raw() & 0x07fc,
                    write: false,
                })
            }
            4 => {
                self.dma = Some(Dma {
                    pif_addr: mask.raw() & 0x07fc,
                    write: true,
                })
            }
            6 => self.rcp_int.clear(RcpIntType::SI),
            _ => todo!("SI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }

    pub fn read_pif<T: Size>(&self, address: u32) -> T {
        self.pif.read(address)
    }

    pub fn write_pif<T: Size>(&mut self, address: u32, value: T) {
        self.pif.write(&mut self.joybus, address, value);
        self.rcp_int.raise(RcpIntType::SI);
    }
}
