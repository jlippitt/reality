pub use joybus::JoypadState;

use crate::interrupt::{RcpIntType, RcpInterrupt};
use crate::memory::{Size, WriteMask};
use crate::rdram::Rdram;
use crc::Crc;
use joybus::Joybus;
use pif::Pif;
use regs::Regs;
use tracing::{debug, error};

mod joybus;
mod pif;
mod regs;

struct Dma {
    pif_addr: u32,
    write: bool,
}

struct Cic {
    variant: u32,
    seed: u32,
    rdram_size_addr: Option<u32>,
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

    pub fn update_joypads(&mut self, joypads: &[JoypadState; 4]) {
        self.joybus.update_joypads(joypads);
    }

    pub fn step(&mut self, rdram: &mut Rdram) {
        if let Some(dma) = &self.dma {
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

    pub fn cic_detect(&mut self, rom_data: &[u8], rdram: &mut Rdram) {
        let ipl3_checksum = Crc::<u32>::new(&crc::CRC_32_CKSUM).checksum(&rom_data[0x0040..0x1000]);
        debug!("IPL3 Checksum: {:08X}", ipl3_checksum);

        // This data is written by PIF upon startup
        // (byte 1 is the seed for the IPL3 CRC check)
        let cic_result = match ipl3_checksum {
            0x0013579c => Some(Cic {
                variant: 6101,
                seed: 0x0004_3f3f,
                rdram_size_addr: Some(0x0318),
            }),
            0xd1f2d592 => Some(Cic {
                variant: 6102,
                seed: 0x0000_3f3f,
                rdram_size_addr: Some(0x0318),
            }),
            0x27df61e2 => Some(Cic {
                variant: 6103,
                seed: 0x0000_783f,
                rdram_size_addr: None,
            }),
            0x229f516c => Some(Cic {
                variant: 6105,
                seed: 0x0000_913f,
                rdram_size_addr: Some(0x03f0),
            }),
            0xa0dd69f7 => Some(Cic {
                variant: 6106,
                seed: 0x0000_853f,
                rdram_size_addr: None,
            }),
            _ => None,
        };

        if let Some(cic) = cic_result {
            debug!("CIC Type: NUS-{}", cic.variant);
            self.pif.write(0x07e4, cic.seed);

            if let Some(address) = cic.rdram_size_addr {
                rdram.write_single(address as usize, 0x0080_0000u32);
            }
        } else {
            error!(
                "IPL3 checksum {:08X} not matched. Could not detect CIC type.",
                ipl3_checksum
            );
        }
    }
}
