use super::cp0;
use super::{Cpu, DcOperation};
use tracing::trace;

pub fn cop2(cpu: &mut Cpu, _pc: u32, word: u32) -> DcOperation {
    if cpu.cp0.cp2_usable() {
        match (word >> 21) & 31 {
            0o00 | 0x01 | 0x02 | 0x04 | 0x05 | 0x06 => (),
            _ => cp0::except(
                cpu,
                cp0::Exception::ReservedInstruction(2),
                cp0::ExceptionStage::EX,
            ),
        }
    } else {
        cp0::except(
            cpu,
            cp0::Exception::CoprocessorUnusable(2),
            cp0::ExceptionStage::EX,
        );
    }

    DcOperation::Nop
}

pub fn syscall(cpu: &mut Cpu, pc: u32) -> DcOperation {
    trace!("{:08X}: SYSCALL", pc,);
    cp0::except(cpu, cp0::Exception::Syscall, cp0::ExceptionStage::EX);
    DcOperation::Nop
}

pub fn break_(cpu: &mut Cpu, pc: u32) -> DcOperation {
    trace!("{:08X}: BREAK", pc,);
    cp0::except(cpu, cp0::Exception::Breakpoint, cp0::ExceptionStage::EX);
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
