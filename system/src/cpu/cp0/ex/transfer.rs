use super::Cp0;
use super::{Cpu, DcState};
use tracing::trace;

pub fn mtc0(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: MTC0 {}, {}",
        pc,
        Cpu::REG_NAMES[rt],
        Cp0::REG_NAMES[rd]
    );

    DcState::Cp0RegWrite {
        reg: rd,
        value: cpu.regs[rt],
    }
}

pub fn mfc0(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: MFC0 {}, {}",
        pc,
        Cpu::REG_NAMES[rt],
        Cp0::REG_NAMES[rd]
    );

    DcState::RegWrite {
        reg: rt,
        value: cpu.cp0.read_reg(rd),
    }
}
