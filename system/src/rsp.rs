use super::memory::{Memory, Size, WriteMask};
use regs::{Regs, Status};
use tracing::{debug, warn};

mod regs;

const MEM_SIZE: usize = 8192;

pub struct Rsp {
    mem: Memory<u128>,
    regs: Regs,
    pc: u32,
}

impl Rsp {
    pub fn new(ipl3_data: Option<&[u8]>) -> Self {
        let mem = if let Some(ipl3_data) = ipl3_data {
            let mut vec = Vec::from(ipl3_data);
            vec.resize(MEM_SIZE, 0);
            Memory::from_bytes(&vec)
        } else {
            Memory::with_byte_len(MEM_SIZE)
        };

        Self {
            mem,
            regs: Regs::default(),
            pc: 0,
        }
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        if (address as usize) < MEM_SIZE {
            return self.mem.read(address as usize);
        }

        T::from_u32(if (address & 0x0004_0000) == 0x0004_0000 {
            self.read_register(address)
        } else if address == 0x0008_0000 {
            self.pc
        } else {
            panic!("Read from unmapped RSP address: {:08X}", address);
        })
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) {
        if (address as usize) < MEM_SIZE {
            return self.mem.write(address as usize, value);
        }

        let mask = WriteMask::new(address, value);

        if (address & 0x0004_0000) == 0x0004_0000 {
            self.write_register(address, mask);
        } else if address == 0x0008_0000 {
            mask.write_partial(&mut self.pc, 0x0000_0ffc);
            debug!("RSP PC: {:08X}", self.pc);
        } else {
            panic!("Write to unmapped RSP address: {:08X}", address);
        }
    }

    fn read_register(&self, address: u32) -> u32 {
        match (address & 0xffff) >> 2 {
            4 => self.regs.status.into(),
            6 => self.regs.status.dma_busy() as u32,
            _ => todo!("RSP Register Read: {:08X}", address),
        }
    }

    fn write_register(&mut self, address: u32, mask: WriteMask) {
        match (address & 0xffff) >> 2 {
            0 => mask.write_reg_hex("SP_DMA_SPADDR", &mut self.regs.dma_sp_addr),
            1 => mask.write_reg_hex("SP_DMA_RAMADDR", &mut self.regs.dma_ram_addr),
            4 => {
                let raw = mask.raw();

                if (raw & 0x0000_0002) != 0 {
                    self.regs.status.set_broke(false);
                }

                if (raw & 0x0000_0008) != 0 {
                    warn!("TODO: Acknowledge RSP Interrupt");
                }

                if (raw & 0x0000_0010) != 0 {
                    todo!("Trigger RSP interrupt");
                }

                mask.set_or_clear(&mut self.regs.status, Status::set_halted, 1, 0);
                mask.set_or_clear(&mut self.regs.status, Status::set_sstep, 6, 5);
                mask.set_or_clear(&mut self.regs.status, Status::set_intbreak, 8, 7);
                mask.set_or_clear(&mut self.regs.status, Status::set_sig0, 10, 9);
                mask.set_or_clear(&mut self.regs.status, Status::set_sig1, 12, 11);
                mask.set_or_clear(&mut self.regs.status, Status::set_sig2, 14, 13);
                mask.set_or_clear(&mut self.regs.status, Status::set_sig3, 16, 15);
                mask.set_or_clear(&mut self.regs.status, Status::set_sig4, 18, 17);
                mask.set_or_clear(&mut self.regs.status, Status::set_sig5, 20, 19);
                mask.set_or_clear(&mut self.regs.status, Status::set_sig6, 22, 21);
                mask.set_or_clear(&mut self.regs.status, Status::set_sig7, 24, 23);

                debug!("SP_regs.status: {:?}", self.regs.status);
            }
            _ => todo!("RSP Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }
}
