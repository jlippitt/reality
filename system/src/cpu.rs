use bytemuck::Pod;
use std::mem;

const COLD_RESET_VECTOR: u32 = 0xbfc0_0000;

pub trait Bus {
    fn read_single<T: Pod>(&self, address: u32) -> T;
}

pub struct Cpu {
    pc: u32,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            pc: COLD_RESET_VECTOR,
        }
    }

    pub fn step(&mut self, bus: &mut impl Bus) {
        let opcode: u32 = self.read(bus, self.pc);
        println!("{:08X}: {1:08X} = {1:032b}", self.pc, opcode);
    }

    fn read<T: Pod>(&self, bus: &mut impl Bus, address: u32) -> T {
        let segment = address >> 29;

        if (segment & 6) != 4 {
            todo!("TLB lookups");
        }

        if segment == 4 {
            todo!("Cached reads");
        }

        if mem::size_of::<T>() > 4 {
            todo!("Block reads");
        }

        bus.read_single(address & 0x1fff_ffff)
    }
}
