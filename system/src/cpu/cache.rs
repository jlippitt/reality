use crate::memory::Memory;
use std::array;
use tracing::trace;

#[cfg(feature = "dcache")]
use crate::memory::Size;

#[derive(Clone, Default, Debug)]
pub struct ICacheLine {
    data: Memory<[u8; 32]>,
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

    pub fn line_mut(&mut self, address: u32) -> &mut ICacheLine {
        let index = ((address >> 5) & 0x01ff) as usize;
        &mut self.lines[index]
    }

    pub fn find_mut(&mut self, vaddr: u32, paddr: u32) -> Option<&mut ICacheLine> {
        let index = ((vaddr >> 5) & 0x01ff) as usize;
        let line = &mut self.lines[index];
        line.matches(paddr).then_some(line)
    }

    pub fn read(&mut self, vaddr: u32, paddr: u32, mut reload: impl FnMut(&mut ICacheLine)) -> u32 {
        // TODO: ITLB?
        let index = ((vaddr >> 5) & 0x01ff) as usize;
        let line = &mut self.lines[index];

        if !line.valid || line.ptag != (paddr >> 12) {
            reload(line);
            line.ptag = paddr >> 12;
            line.valid = true;
            trace!("ICache Line {}: {:08X?}", index, line);
        }

        line.data.read(vaddr as usize & 0x1f)
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
    pub fn matches(&self, address: u32) -> bool {
        self.valid && self.ptag == (address >> 12)
    }

    pub fn bytes_mut(&mut self) -> &mut [u8] {
        self.data.as_bytes_mut()
    }

    pub fn clear_valid_flag(&mut self) {
        self.valid = false;
    }
}

#[cfg(feature = "dcache")]
#[derive(Clone, Default, Debug)]
pub struct DCacheLine {
    data: Memory<[u8; 16]>,
    ptag: u32,
    valid: bool,
    dirty: bool,
}

#[cfg(feature = "dcache")]
pub struct DCache {
    lines: [DCacheLine; 512],
}

#[cfg(feature = "dcache")]
impl DCache {
    pub fn new() -> Self {
        Self {
            lines: array::from_fn(|_| Default::default()),
        }
    }

    pub fn find_mut(&mut self, address: u32) -> Option<&mut DCacheLine> {
        let index = ((address >> 4) & 0x01ff) as usize;
        let line = &mut self.lines[index];
        line.matches(address).then_some(line)
    }

    pub fn read<T: Size>(&mut self, address: u32, reload: impl FnMut(&mut DCacheLine)) -> T {
        let line = self.fetch_line(address, reload);
        line.data.read(address as usize & 0x0f)
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T, reload: impl FnMut(&mut DCacheLine)) {
        let line = self.fetch_line(address, reload);
        line.data.write(address as usize & 0x0f, value);
        line.dirty = true;
    }

    pub fn index_write_back_invalidate(
        &mut self,
        address: u32,
        mut store: impl FnMut(&DCacheLine),
    ) {
        let index = ((address >> 4) & 0x01ff) as usize;
        let line = &mut self.lines[index];

        if line.is_dirty() {
            store(line);
        }

        line.valid = false;
        trace!("DCache Line {} Invalidated", index);
    }

    pub fn index_store_tag(&mut self, address: u32, ptag: u32, valid: bool, dirty: bool) {
        let index = ((address >> 4) & 0x01ff) as usize;
        let line = &mut self.lines[index];
        line.ptag = ptag;
        line.valid = valid;
        line.dirty = dirty;
    }

    pub fn create_dirty_exclusive(&mut self, address: u32, mut store: impl FnMut(&DCacheLine)) {
        let index = ((address >> 4) & 0x01ff) as usize;
        let line = &mut self.lines[index];

        if line.is_dirty() && line.ptag() != (address >> 12) {
            store(line);
        }

        line.ptag = address >> 20;
        line.valid = true;
        line.dirty = true;
        trace!("DCache Line {}: {:08X?}", index, line);
    }

    pub fn hit_write_back_invalidate(&mut self, address: u32, mut store: impl FnMut(&DCacheLine)) {
        let index = ((address >> 4) & 0x01ff) as usize;
        let line = &mut self.lines[index];

        if line.matches(address) {
            if line.is_dirty() {
                store(line);
            }

            line.valid = false;
            trace!("DCache Line {} Invalidated", index);
        }
    }

    fn fetch_line(
        &mut self,
        address: u32,
        mut reload: impl FnMut(&mut DCacheLine),
    ) -> &mut DCacheLine {
        let index = ((address >> 4) & 0x01ff) as usize;
        let line = &mut self.lines[index];

        if !line.matches(address) {
            reload(line);
            line.ptag = address >> 12;
            line.valid = true;
            line.dirty = false;
            trace!("DCache Line {}: {:08X?}", index, line);
        }

        line
    }
}

#[cfg(feature = "dcache")]
impl DCacheLine {
    pub fn matches(&self, address: u32) -> bool {
        self.valid && self.ptag == (address >> 12)
    }

    pub fn bytes(&self) -> &[u8] {
        self.data.as_bytes()
    }

    pub fn bytes_mut(&mut self) -> &mut [u8] {
        self.data.as_bytes_mut()
    }

    pub fn ptag(&self) -> u32 {
        self.ptag
    }

    pub fn is_dirty(&self) -> bool {
        self.valid && self.dirty
    }

    pub fn clear_valid_flag(&mut self) {
        self.valid = false;
    }

    pub fn clear_dirty_flag(&mut self) {
        self.dirty = false;
    }
}
