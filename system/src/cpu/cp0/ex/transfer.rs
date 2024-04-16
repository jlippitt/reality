use super::Cp0;
use super::{Cpu, DcOperation};
use tracing::trace;

pub fn mfc0(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: MFC0 {}, {}",
        pc,
        Cpu::REG_NAMES[rt],
        Cp0::REG_NAMES[rd]
    );

    DcOperation::RegWrite {
        reg: rt,
        value: cpu.cp0.read_reg(rd),
    }
}

pub fn dmfc0(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: DMFC0 {}, {}",
        pc,
        Cpu::REG_NAMES[rt],
        Cp0::REG_NAMES[rd]
    );

    DcOperation::RegWrite {
        reg: rt,
        value: cpu.cp0.read_reg(rd),
    }
}

pub fn mtc0(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: MTC0 {}, {}",
        pc,
        Cpu::REG_NAMES[rt],
        Cp0::REG_NAMES[rd]
    );

    DcOperation::Cp0RegWrite {
        reg: rd,
        value: cpu.regs[rt],
    }
}

pub fn dmtc0(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: DMTC0 {}, {}",
        pc,
        Cpu::REG_NAMES[rt],
        Cp0::REG_NAMES[rd]
    );

    DcOperation::Cp0RegWrite {
        reg: rd,
        value: cpu.regs[rt],
    }
}
