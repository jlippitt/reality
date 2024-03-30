use super::memory::Memory;
use crate::cpu::Size;
use regs::Status;

mod regs;

const MEM_SIZE: usize = 8192;

pub struct Rsp {
    mem: Memory,
    status: Status,
}

impl Rsp {
    pub fn new() -> Self {
        Self {
            mem: Memory::new(MEM_SIZE),
            status: Status::new().with_halted(true),
        }
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        if (address as usize) < MEM_SIZE {
            self.mem.read(address)
        } else if address >= 0x0004_0000 {
            // Read register
            T::from_u32(self.read_register(address))
        } else {
            panic!("Read from unmapped RSP address: {:08X}", address);
        }
    }

    fn read_register(&self, address: u32) -> u32 {
        match (address & 0xffff) >> 2 {
            4 => self.status.into(),
            _ => todo!("RSP Register Read: {:08X}", address),
        }
    }
}
