use super::{Cpu, DcState};
use tracing::trace;

pub fn sw(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let base = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let offset = (word & 0xffff) as i16 as i64;

    trace!(
        "{:08X}: SW {}, {}({})",
        pc,
        Cpu::REG_NAMES[rt],
        offset,
        Cpu::REG_NAMES[base],
    );

    DcState::StoreWord {
        value: cpu.regs[rt],
        addr: cpu.regs[base].wrapping_add(offset) as u32,
    }
}
