use super::{Cpu, DcState};
use tracing::trace;

pub fn mult(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;

    trace!(
        "{:08X}: MULT {}, {}",
        pc,
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
    );

    let result = (cpu.regs[rs] as i32) as i64 * (cpu.regs[rt] as i32) as i64;

    cpu.hi = result >> 32;
    trace!("  HI: {:016X}", cpu.hi);

    cpu.lo = (result as i32) as i64;
    trace!("  LO: {:016X}", cpu.lo);

    DcState::Nop
}

pub fn multu(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;

    trace!(
        "{:08X}: MULTU {}, {}",
        pc,
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
    );

    let result = (cpu.regs[rs] as u32) as u64 * (cpu.regs[rt] as u32) as u64;

    cpu.hi = (result as i64) >> 32;
    trace!("  HI: {:016X}", cpu.hi);

    cpu.lo = (result as u32) as i32 as i64;
    trace!("  LO: {:016X}", cpu.lo);

    DcState::Nop
}

pub fn mfhi(_cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rd = ((word >> 11) & 31) as usize;
    trace!("{:08X}: MFHI {}", pc, Cpu::REG_NAMES[rd],);
    DcState::MfHi { reg: rd }
}

pub fn mflo(_cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rd = ((word >> 11) & 31) as usize;
    trace!("{:08X}: MFLO {}", pc, Cpu::REG_NAMES[rd],);
    DcState::MfLo { reg: rd }
}
