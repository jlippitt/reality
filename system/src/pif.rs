use super::memory::Memory;
use bytemuck::Pod;

const PIF_DATA_SIZE: usize = 2048;

pub struct Pif {
    mem: Memory,
}

impl Pif {
    pub fn new(data: Vec<u8>) -> Self {
        assert!(data.len() == PIF_DATA_SIZE);
        let mut mem: Memory = data.into();
        mem.write(0x7e4, 0u32);
        mem.write(0x7ff, 0u8);
        Self { mem }
    }

    pub fn read<T: Pod>(&self, address: u32) -> T {
        self.mem.read(address)
    }

    pub fn write<T: Pod>(&mut self, address: u32, value: T) {
        if address < 0x7c0 {
            panic!("Write to PIF ROM: {:08X}", address);
        }

        self.mem.write(address, value)
    }
}
