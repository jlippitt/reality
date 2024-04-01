use tracing::trace;

#[derive(Copy, Clone, Default, Debug)]
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
            lines: [Default::default(); 512],
        }
    }

    pub fn index_store_tag(&mut self, address: u32, ptag: u32, valid: bool) {
        let index = ((address >> 5) & 0x01ff) as usize;
        let line = &mut self.lines[index];
        line.ptag = ptag;
        line.valid = valid;
        trace!("ICache Line {}: {:?}", index, line);
    }
}

#[derive(Copy, Clone, Default, Debug)]
struct DCacheLine {
    data: [u32; 4],
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
            lines: [Default::default(); 512],
        }
    }

    pub fn index_store_tag(&mut self, address: u32, ptag: u32, valid: bool, dirty: bool) {
        let index = ((address >> 4) & 0x01ff) as usize;
        let line = &mut self.lines[index];
        line.ptag = ptag;
        line.valid = valid;
        line.dirty = dirty;
        trace!("DCache Line {}: {:?}", index, line);
    }
}
