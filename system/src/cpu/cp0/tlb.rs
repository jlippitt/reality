use super::regs::{EntryHi, EntryLo, PageMask, Regs};
use std::slice::Iter;
use tracing::trace;

#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct TlbEntry {
    pub entry_lo0: EntryLo,
    pub entry_lo1: EntryLo,
    pub entry_hi: EntryHi,
    pub page_mask: PageMask,
}

#[derive(Debug)]
pub struct Tlb {
    entries: [TlbEntry; 32],
}

impl Tlb {
    pub fn new() -> Self {
        Self {
            entries: Default::default(),
        }
    }

    pub fn entries(&self) -> Iter<TlbEntry> {
        self.entries.iter()
    }

    pub fn read_entry(&self, regs: &mut Regs, index: usize) {
        let entry = &self.entries[index];

        let entry_hi =
            EntryHi::from(u64::from(entry.entry_hi) & !(u32::from(entry.page_mask) as u64));
        let global = entry.entry_hi.global();

        regs.entry_lo0 = entry.entry_lo0.with_global(global);
        regs.entry_lo1 = entry.entry_lo1.with_global(global);
        regs.entry_hi = entry_hi.with_global(false);
        regs.page_mask = entry.page_mask;

        trace!("  EntryLo0: {:?}", regs.entry_lo0);
        trace!("  EntryLo1: {:?}", regs.entry_lo1);
        trace!("  EntryHi: {:?}", regs.entry_hi);
        trace!("  PageMask: {:?}", regs.page_mask);
    }

    pub fn write_entry(&mut self, regs: &Regs) {
        let index = regs.index.index() as usize;
        assert!(index < self.entries.len());

        let entry_hi =
            EntryHi::from(u64::from(regs.entry_hi) & !(u32::from(regs.page_mask) as u64));
        let global = regs.entry_lo0.global() & regs.entry_lo1.global();

        self.entries[index] = TlbEntry {
            entry_lo0: (u32::from(regs.entry_lo0) & 0x03ff_ffff).into(),
            entry_lo1: (u32::from(regs.entry_lo1) & 0x03ff_ffff).into(),
            entry_hi: entry_hi.with_global(global),
            page_mask: regs.page_mask,
        };

        trace!("  TLB{}: {:?}", index, self.entries[index]);
    }
}
