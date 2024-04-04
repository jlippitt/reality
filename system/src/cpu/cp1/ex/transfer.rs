use super::LoadOperator;
use super::StoreOperator;
use super::{Cpu, DcState, Format};
use tracing::trace;

pub struct Lwc1;
pub struct Ldc1;
pub struct Swc1;
pub struct Sdc1;

impl LoadOperator for Lwc1 {
    const NAME: &'static str = "LWC1";

    fn apply(reg: usize, addr: u32) -> DcState {
        DcState::Cp1LoadWord { reg, addr }
    }
}

impl LoadOperator for Ldc1 {
    const NAME: &'static str = "LDC1";

    fn apply(reg: usize, addr: u32) -> DcState {
        DcState::Cp1LoadDoubleword { reg, addr }
    }
}

impl StoreOperator for Swc1 {
    const NAME: &'static str = "SWC1";

    fn apply(cpu: &Cpu, reg: usize, addr: u32) -> DcState {
        DcState::StoreWord {
            value: i32::cp1_reg(cpu, reg) as u32,
            addr,
        }
    }
}

impl StoreOperator for Sdc1 {
    const NAME: &'static str = "SDC1";

    fn apply(cpu: &Cpu, reg: usize, addr: u32) -> DcState {
        DcState::StoreDoubleword {
            value: i64::cp1_reg(cpu, reg) as u64,
            addr,
        }
    }
}

pub fn mfc1(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!("{:08X}: MFC1 {}, F{}", pc, Cpu::REG_NAMES[rt], rd,);

    DcState::RegWrite {
        reg: rt,
        value: i32::cp1_reg(cpu, rd) as i64,
    }
}

pub fn dmfc1(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!("{:08X}: DMFC1 {}, F{}", pc, Cpu::REG_NAMES[rt], rd,);

    DcState::RegWrite {
        reg: rt,
        value: i64::cp1_reg(cpu, rd),
    }
}

pub fn mtc1(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!("{:08X}: MTC1 {}, F{}", pc, Cpu::REG_NAMES[rt], rd,);

    i32::set_cp1_reg(cpu, rd, cpu.regs[rt] as i32).into()
}

pub fn dmtc1(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!("{:08X}: DMTC1 {}, F{}", pc, Cpu::REG_NAMES[rt], rd,);

    i64::set_cp1_reg(cpu, rd, cpu.regs[rt]).into()
}
