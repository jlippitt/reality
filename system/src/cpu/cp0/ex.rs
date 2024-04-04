use super::regs::Status;
use super::Cp0Register;
use super::{Cpu, DcState};
use tracing::trace;

pub fn mtc0(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = Cp0Register::from((word >> 11) & 31);

    trace!("{:08X}: MTC0 {}, {:?}", pc, Cpu::REG_NAMES[rt], rd);

    DcState::Cp0Write {
        reg: rd,
        value: cpu.regs[rt],
    }
}

pub fn mfc0(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = Cp0Register::from((word >> 11) & 31);

    trace!("{:08X}: MFC0 {}, {:?}", pc, Cpu::REG_NAMES[rt], rd);

    DcState::RegWrite {
        reg: rt,
        value: cpu.cp0.read_reg(rd),
    }
}

pub fn eret(cpu: &mut Cpu, pc: u32) -> DcState {
    trace!("{:08X}: ERET", pc);

    let mut status = Status::from(cpu.cp0.regs[Cp0Register::Status as usize] as u32);

    if status.erl() {
        cpu.pc = cpu.cp0.regs[Cp0Register::ErrorEPC as usize] as u32;
        status.set_erl(false);
    } else {
        cpu.pc = cpu.cp0.regs[Cp0Register::EPC as usize] as u32;
        status.set_exl(false);
    }

    cpu.cp0.regs[Cp0Register::Status as usize] = u32::from(status) as i64;
    cpu.cp0.ll_bit = false;
    cpu.rf.word = 0;

    DcState::Nop
}
