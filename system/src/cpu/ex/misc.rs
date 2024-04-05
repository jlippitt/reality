use super::{Cpu, DcState};
use tracing::trace;

pub fn sync(_cpu: &mut Cpu, pc: u32) -> DcState {
    trace!("{:08X}: SYNC", pc);
    // This is a NOP on the VR4300
    DcState::Nop
}

pub fn cache(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    const CACHE_OP_NAMES: [char; 8] = ['?', '?', 'P', '?', '?', '?', '?', '?'];
    const CACHE_NAMES: [char; 4] = ['I', 'D', '?', '?'];

    let base = ((word >> 21) & 31) as usize;
    let op = (word >> 16) & 31;
    let offset = (word & 0xffff) as i16;

    trace!(
        "{:08X}: CACHE {}{}, {}({})",
        pc,
        CACHE_OP_NAMES[(op >> 2) as usize],
        CACHE_NAMES[(op & 3) as usize],
        offset,
        Cpu::REG_NAMES[base]
    );

    let address = cpu.regs[base].wrapping_add(offset as i64) as u32;

    DcState::CacheOperation { op, vaddr: address }
}
