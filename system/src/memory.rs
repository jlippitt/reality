use bytemuck::Pod;
use std::mem;

#[derive(Copy, Clone, Debug)]
pub enum Mapping {
    None,
    Pif,
}

pub struct Memory {
    vec: Vec<u32>,
}

impl Memory {
    pub fn read<T: Pod>(&self, address: u32) -> T {
        let mem_size = mem::size_of::<u32>();
        let data_size = mem::size_of::<T>();
        assert!((mem_size % data_size) == 0);
        let index = address as usize >> data_size.ilog2();
        let slice: &[T] = bytemuck::must_cast_slice(&self.vec);
        slice[index ^ ((mem_size / data_size) - 1)]
    }
}

impl From<Vec<u8>> for Memory {
    fn from(value: Vec<u8>) -> Self {
        let vec = value
            .chunks_exact(4)
            .map(|chunks| u32::from_be_bytes([chunks[0], chunks[1], chunks[2], chunks[3]]))
            .collect();

        Self { vec }
    }
}
