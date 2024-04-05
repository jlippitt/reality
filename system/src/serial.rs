use crate::interrupt::{RcpIntType, RcpInterrupt};
use crate::memory::{Size, WriteMask};
use pif::Pif;
use regs::Regs;

mod pif;
mod regs;

pub struct SerialInterface {
    regs: Regs,
    pif: Pif,
    rcp_int: RcpInterrupt,
}

impl SerialInterface {
    pub fn new(rcp_int: RcpInterrupt, pif_data: Vec<u8>) -> Self {
        Self {
            regs: Regs::default(),
            pif: Pif::new(pif_data),
            rcp_int,
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
            6 => self.rcp_int.clear(RcpIntType::SI),
            _ => todo!("SI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }

    pub fn read_pif<T: Size>(&self, address: u32) -> T {
        self.pif.read(address)
    }

    pub fn write_pif<T: Size>(&mut self, address: u32, value: T) {
        self.pif.write(address, value);
        self.rcp_int.raise(RcpIntType::SI);
    }
}
