use super::memory::Memory;
use bytemuck::Pod;

const PIF_DATA_SIZE: usize = 2048;

pub struct Pif {
    data: Memory,
}

impl Pif {
    pub fn new(data: Vec<u8>) -> Self {
        assert!(data.len() == PIF_DATA_SIZE);
        Self { data: data.into() }
    }

    pub fn read<T: Pod>(&self, address: u32) -> T {
        self.data.read(address)
    }
}
