use super::cp0;
use super::{Cpu, DcOperation};
use tracing::trace;

pub fn cop2_unusable(cpu: &mut Cpu) -> DcOperation {
    cp0::except(cpu, cp0::Exception::CoprocessorUnusable(2));
    DcOperation::Nop
}

pub fn syscall(cpu: &mut Cpu, pc: u32) -> DcOperation {
    trace!("{:08X}: SYSCALL", pc,);
    cp0::except(cpu, cp0::Exception::Syscall);
    DcOperation::Nop
}

pub fn break_(cpu: &mut Cpu, pc: u32) -> DcOperation {
    trace!("{:08X}: BREAK", pc,);
    cp0::except(cpu, cp0::Exception::Breakpoint);
    DcOperation::Nop
}

pub fn teq(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;

    trace!(
        "{:08X}: TEQ {}, {}",
        pc,
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
    );

    if cpu.regs[rs] == cpu.regs[rt] {
        todo!("TrapException");
    }

    DcOperation::Nop
}

pub fn tne(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;

    trace!(
        "{:08X}: TNE {}, {}",
        pc,
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
    );

    if cpu.regs[rs] != cpu.regs[rt] {
        todo!("TrapException");
    }

    DcOperation::Nop
}
