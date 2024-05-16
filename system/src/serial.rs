pub use joybus::JoypadState;

use crate::header::{CicType, SaveType};
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
    pub fn new(
        rcp_int: RcpInterrupt,
        pif_data: Option<Vec<u8>>,
        cic_type: CicType,
        save_type: SaveType,
    ) -> Self {
        let mut pif = Pif::new(pif_data);

        let cic_seed: Option<u32> = match cic_type {
            CicType::Nus6101 => Some(0x0004_3f3f),
            CicType::Nus6102 | CicType::MiniIPL3 => Some(0x0000_3f3f),
            CicType::Nus6103 => Some(0x0000_783f),
            CicType::Nus6105 => Some(0x0000_913f),
            CicType::Nus6106 => Some(0x0000_853f),
            _ => None,
        };

        if let Some(seed) = cic_seed {
            pif.write(0x07e4, seed);
        }

        Self {
            regs: Regs::default(),
            joybus: Joybus::new(save_type),
            pif,
            dma: None,
            rcp_int,
        }
    }

    pub fn update_joypads(&mut self, joypads: &[JoypadState; 4]) {
        self.joybus.update_joypads(joypads);
    }

    #[inline(always)]
    pub fn step(&mut self, rdram: &mut Rdram) {
        if self.dma.is_none() {
            return;
        }

        self.step_inner(rdram);
    }

    fn step_inner(&mut self, rdram: &mut Rdram) {
        let dma = self.dma.as_ref().unwrap();

        let dram_addr = self.regs.dram_addr.dram_addr();
        let mut buf = [0u8; 64];

        if dma.write {
            rdram.read_block(dram_addr as usize, &mut buf);

            let mut pif_addr = dma.pif_addr;

            let mut joybus_configure = false;

            for byte in buf {
                joybus_configure |= self.pif.write(pif_addr, byte);
                // TODO: Can this wrap?
                pif_addr += 1;
            }

            debug!(
                "SI DMA: {} bytes written from {:08X} to {:04X}",
                buf.len() * 4,
                dram_addr,
                dma.pif_addr,
            );

            if joybus_configure {
                self.joybus.configure(self.pif.ram());
            }
        } else {
            self.joybus.execute(self.pif.ram_mut());

            let mut pif_addr = dma.pif_addr;

            for byte in &mut buf {
                *byte = self.pif.read(pif_addr);
                // TODO: Can this wrap?
                pif_addr += 1;
            }

            rdram.write_block(dram_addr as usize, &buf);

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

    pub fn read<T: Size>(&self, address: u32) -> T {
        T::truncate_u32(match address >> 2 {
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
        if self.pif.write(address, value) {
            self.joybus.configure(self.pif.ram());
        }

        self.rcp_int.raise(RcpIntType::SI);
    }
}
