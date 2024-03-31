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
}
