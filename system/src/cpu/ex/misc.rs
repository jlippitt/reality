use super::{Cpu, DcOperation};
use tracing::trace;

pub fn sync(_cpu: &mut Cpu, pc: u32) -> DcOperation {
    trace!("{:08X}: SYNC", pc);
    // This is a NOP on the VR4300
    DcOperation::Nop
}

pub fn cache(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let base = ((word >> 21) & 31) as usize;
    let op = (word >> 16) & 31;
    let offset = (word & 0xffff) as i16;

    trace!(
        "{:08X}: CACHE 0b{:05b}, {}({})",
        pc,
        op,
        offset,
        Cpu::REG_NAMES[base]
    );

    let address = cpu.regs[base].wrapping_add(offset as i64) as u32;

    DcOperation::CacheOperation { op, vaddr: address }
}
