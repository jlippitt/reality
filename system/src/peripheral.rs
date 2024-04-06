use crate::interrupt::{RcpIntType, RcpInterrupt};
use crate::memory::{Memory, Size, WriteMask};
use crate::rdram::Rdram;
use regs::Regs;
use tracing::{debug, warn};

mod regs;

struct Dma {
    len: u32,
    write: bool,
}

pub struct PeripheralInterface {
    regs: Regs,
    rom: Memory,
    dma: Option<Dma>,
    rcp_int: RcpInterrupt,
}

impl PeripheralInterface {
    pub fn new(rcp_int: RcpInterrupt, rom_data: Vec<u8>, skip_pif_rom: bool) -> Self {
        let mut regs = Regs::default();

        if skip_pif_rom {
            regs.bsd_dom[0].lat.set_lat(rom_data[0] as u32);
            regs.bsd_dom[0].pwd.set_pwd(rom_data[1] as u32);
            regs.bsd_dom[0].pgs.set_pgs(rom_data[2] as u32 & 0x0f);
            regs.bsd_dom[0].rls.set_rls(rom_data[1] as u32 >> 8);
        }

        Self {
            regs,
            rom: Memory::from_bytes(&rom_data),
            dma: None,
            rcp_int,
        }
    }

    pub fn step(&mut self, rdram: &mut Rdram) {
        if let Some(dma) = &mut self.dma {
            let block_address = self.regs.dram_addr;
            let block_len = dma.len.min(128);

            assert!((block_address & 3) == 0);
            assert!((block_len & 3) == 0);

            let mut buf = [0u32; 32];
            let data = &mut buf[0..((block_len >> 2) as usize)];

            if dma.write {
                self.rom.read_block(self.regs.cart_addr & 0x00ff_ffff, data);
                rdram.write_block(self.regs.dram_addr & 0x00ff_ffff, data);

                debug!(
                    "PI DMA: {} bytes written from {:08X} to {:08X}",
                    block_len, self.regs.cart_addr, self.regs.dram_addr,
                );
            } else {
                rdram.read_block(self.regs.dram_addr, data);
                self.rom.write_block(self.regs.cart_addr, data);

                debug!(
                    "PI DMA: {} bytes read from {:08X} to {:08X}",
                    block_len, self.regs.dram_addr, self.regs.cart_addr,
                );
            }

            // TODO: Can these wrap?
            self.regs.dram_addr += block_len;
            self.regs.cart_addr += block_len;
            dma.len -= block_len;

            if dma.len == 0 {
                self.dma = None;
                self.rcp_int.raise(RcpIntType::PI);
            }
        }
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        T::from_u32(match address >> 2 {
            0 => self.regs.dram_addr,
            1 => self.regs.cart_addr,
            4 => {
                let mut value: u32 = 0;

                if self.dma.is_some() {
                    value |= 0x01;
                }

                if self.rcp_int.has(RcpIntType::PI) {
                    value |= 0x08;
                }

                value
            }
            5 => self.regs.bsd_dom[0].lat.into(),
            6 => self.regs.bsd_dom[0].pwd.into(),
            7 => self.regs.bsd_dom[0].pgs.into(),
            8 => self.regs.bsd_dom[0].rls.into(),
            9 => self.regs.bsd_dom[1].lat.into(),
            10 => self.regs.bsd_dom[1].pwd.into(),
            11 => self.regs.bsd_dom[1].pgs.into(),
            12 => self.regs.bsd_dom[1].rls.into(),
            _ => todo!("PI Register Read: {:08X}", address),
        })
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        match address >> 2 {
            0 => mask.write_reg_hex("PI_DRAM_ADDR", &mut self.regs.dram_addr),
            1 => mask.write_reg_hex("PI_CART_ADDR", &mut self.regs.cart_addr),
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
                    self.rcp_int.clear(RcpIntType::PI);
                }
            }
            5 => mask.write_reg("PI_BSD_DOM1_LAT", &mut self.regs.bsd_dom[0].lat),
            6 => mask.write_reg("PI_BSD_DOM1_PWD", &mut self.regs.bsd_dom[0].pwd),
            7 => mask.write_reg("PI_BSD_DOM1_PGS", &mut self.regs.bsd_dom[0].pgs),
            8 => mask.write_reg("PI_BSD_DOM1_RLS", &mut self.regs.bsd_dom[0].rls),
            9 => mask.write_reg("PI_BSD_DOM2_LAT", &mut self.regs.bsd_dom[1].lat),
            10 => mask.write_reg("PI_BSD_DOM2_PWD", &mut self.regs.bsd_dom[1].pwd),
            11 => mask.write_reg("PI_BSD_DOM2_PGS", &mut self.regs.bsd_dom[1].pgs),
            12 => mask.write_reg("PI_BSD_DOM2_RLS", &mut self.regs.bsd_dom[1].rls),
            _ => todo!("PI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }

    pub fn read_rom<T: Size>(&self, address: u32) -> T {
        self.rom.read(address)
    }
}
