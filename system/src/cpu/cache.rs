use crate::memory::{Memory, Size};
use std::array;
use tracing::trace;

#[derive(Clone, Default, Debug)]
struct ICacheLine {
    data: [u32; 8],
    ptag: u32,
    valid: bool,
}

pub struct ICache {
    lines: [ICacheLine; 512],
}

impl ICache {
    pub fn new() -> Self {
        Self {
            lines: array::from_fn(|_| Default::default()),
        }
    }

    pub fn read(&self, address: u32) -> Option<u32> {
        assert_eq!(
            0x8000_0000,
            address & 0xc000_0000,
            "ITLB and ICache banking not implemented"
        );

        let index = ((address >> 5) & 0x01ff) as usize;
        let line = &self.lines[index];

        (line.valid && line.ptag == (address >> 12))
            .then(|| line.data[((address >> 2) & 7) as usize])
    }

    pub fn insert_line(&mut self, address: u32, data: [u32; 8]) -> &mut [u32; 8] {
        assert_eq!(
            0x8000_0000,
            address & 0xc000_0000,
            "ITLB and ICache banking not implemented"
        );

        let index = ((address >> 5) & 0x01ff) as usize;
        let line = &mut self.lines[index];

        line.data = data;
        line.ptag = address >> 12;
        line.valid = true;

        trace!("ICache Line {}: {:08X?}", index, line);

        &mut line.data
    }

    pub fn index_store_tag(&mut self, address: u32, ptag: u32, valid: bool) {
        let index = ((address >> 5) & 0x01ff) as usize;
        let line = &mut self.lines[index];
        line.ptag = ptag;
        line.valid = valid;
        trace!("ICache Line {}: {:08X?}", index, line);
    }
}

#[derive(Clone, Default, Debug)]
struct DCacheLine {
    data: Memory<[u32; 4]>,
    ptag: u32,
    valid: bool,
    dirty: bool,
}

pub struct DCache {
    lines: [DCacheLine; 512],
}

impl DCache {
    pub fn new() -> Self {
        Self {
            lines: array::from_fn(|_| Default::default()),
        }
    }

    pub fn read<T: Size>(&self, address: u32) -> Option<T> {
        assert_eq!(
            0x8000_0000,
            address & 0xc000_0000,
            "TLB not yet implemented"
        );

        let index = ((address >> 4) & 0x01ff) as usize;
        let line = &self.lines[index];

        (line.valid && line.ptag == (address >> 12)).then(|| line.data.read(address & 0x0f))
    }

    pub fn read_block(&self, address: u32, data: &mut [u32]) -> bool {
        assert_eq!(
            0x8000_0000,
            address & 0xc000_0000,
            "TLB not yet implemented"
        );

        let index = ((address >> 4) & 0x01ff) as usize;
        let line = &self.lines[index];

        if line.valid && line.ptag == (address >> 12) {
            line.data.read_block(address & 0x0f, data);
            true
        } else {
            false
        }
    }

    pub fn insert_line(&mut self, address: u32, data: [u32; 4]) -> &mut Memory<[u32; 4]> {
        assert_eq!(
            0x8000_0000,
            address & 0xc000_0000,
            "TLB not yet implemented"
        );

        let index = ((address >> 4) & 0x01ff) as usize;
        let line = &mut self.lines[index];

        if line.dirty {
            todo!("DCache writeback");
        }

        line.data = data.into();
        line.ptag = address >> 12;
        line.valid = true;

        trace!("DCache Line {}: {:08X?}", index, line);

        &mut line.data
    }

    pub fn index_store_tag(&mut self, address: u32, ptag: u32, valid: bool, dirty: bool) {
        let index = ((address >> 4) & 0x01ff) as usize;
        let line = &mut self.lines[index];
        line.ptag = ptag;
        line.valid = valid;
        line.dirty = dirty;
        trace!("DCache Line {}: {:08X?}", index, line);
    }
}
