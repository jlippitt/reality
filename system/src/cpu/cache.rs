use crate::memory::{Memory, Size};
use std::array;
use tracing::trace;

#[derive(Clone, Default, Debug)]
pub struct ICacheLine {
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

    pub fn read(&mut self, address: u32, mut reload: impl FnMut(&mut ICacheLine)) -> u32 {
        // TODO: ITLB?
        let index = ((address >> 5) & 0x01ff) as usize;
        let line = &mut self.lines[index];

        if !line.valid || line.ptag != (address >> 12) {
            reload(line);
            line.ptag = address >> 12;
            line.valid = true;
            trace!("ICache Line {}: {:08X?}", index, line);
        }

        line.data[((address >> 2) & 7) as usize]
    }

    pub fn index_store_tag(&mut self, address: u32, ptag: u32, valid: bool) {
        let index = ((address >> 5) & 0x01ff) as usize;
        let line = &mut self.lines[index];
        line.ptag = ptag;
        line.valid = valid;
        trace!("ICache Line {}: {:08X?}", index, line);
    }
}

impl ICacheLine {
    pub fn data_mut(&mut self) -> &mut [u32] {
        &mut self.data
    }
}

#[derive(Clone, Default, Debug)]
pub struct DCacheLine {
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

    pub fn read<T: Size>(&mut self, address: u32, reload: impl FnMut(&mut DCacheLine)) -> T {
        let line = self.fetch_line(address, reload);
        line.data.read(address & 0x0f)
    }

    pub fn read_block(
        &mut self,
        address: u32,
        data: &mut [u32],
        reload: impl FnMut(&mut DCacheLine),
    ) {
        let line = self.fetch_line(address, reload);
        line.data.read_block(address & 0x0f, data);
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T, reload: impl FnMut(&mut DCacheLine)) {
        let line = self.fetch_line(address, reload);
        line.data.write(address & 0x0f, value);
        line.dirty = true;
    }

    pub fn write_block(&mut self, address: u32, data: &[u32], reload: impl FnMut(&mut DCacheLine)) {
        let line = self.fetch_line(address, reload);
        line.data.write_block(address & 0x0f, data);
        line.dirty = true;
    }

    pub fn index_store_tag(&mut self, address: u32, ptag: u32, valid: bool, dirty: bool) {
        let index = ((address >> 4) & 0x01ff) as usize;
        let line = &mut self.lines[index];
        line.ptag = ptag;
        line.valid = valid;
        line.dirty = dirty;
    }

    fn fetch_line(
        &mut self,
        address: u32,
        mut reload: impl FnMut(&mut DCacheLine),
    ) -> &mut DCacheLine {
        let index = ((address >> 4) & 0x01ff) as usize;
        let line = &mut self.lines[index];

        if !line.valid || line.ptag != (address >> 12) {
            reload(line);
            line.ptag = address >> 12;
            line.valid = true;
            line.dirty = false;
            trace!("DCache Line {}: {:08X?}", index, line);
        }

        line
    }
}

impl DCacheLine {
    pub fn data(&self) -> &[u32] {
        self.data.as_slice()
    }

    pub fn data_mut(&mut self) -> &mut [u32] {
        self.data.as_mut_slice()
    }

    pub fn ptag(&self) -> u32 {
        self.ptag
    }

    pub fn is_dirty(&self) -> bool {
        self.valid && self.dirty
    }
}
