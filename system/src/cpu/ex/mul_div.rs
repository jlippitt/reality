use super::{Cpu, DcState};
use tracing::trace;

pub trait MulDivOperator {
    const NAME: &'static str;
    fn apply(lhs: i64, rhs: i64) -> (i64, i64);
}

pub struct Mult;
pub struct Dmult;
pub struct Multu;
pub struct Dmultu;

impl MulDivOperator for Mult {
    const NAME: &'static str = "MULT";

    fn apply(lhs: i64, rhs: i64) -> (i64, i64) {
        let result = lhs as i32 as i64 * rhs as i32 as i64;
        ((result >> 32), result as i32 as i64)
    }
}

impl MulDivOperator for Dmult {
    const NAME: &'static str = "DMULT";

    fn apply(lhs: i64, rhs: i64) -> (i64, i64) {
        let result = lhs as i128 * rhs as i128;
        ((result >> 64) as i64, result as i64)
    }
}

impl MulDivOperator for Multu {
    const NAME: &'static str = "MULTU";

    fn apply(lhs: i64, rhs: i64) -> (i64, i64) {
        let result = lhs as u32 as u64 * rhs as u32 as u64;
        (((result as i64) >> 32), result as i32 as i64)
    }
}

impl MulDivOperator for Dmultu {
    const NAME: &'static str = "DMULTU";

    fn apply(lhs: i64, rhs: i64) -> (i64, i64) {
        let result = lhs as u64 as u128 * rhs as u64 as u128;
        ((result >> 64) as i64, result as i64)
    }
}

pub fn mul_div<Op: MulDivOperator>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;

    trace!(
        "{:08X}: {} {}, {}",
        pc,
        Op::NAME,
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
    );

    (cpu.hi, cpu.lo) = Op::apply(cpu.regs[rs], cpu.regs[rt]);

    trace!("  HI: {:016X}", cpu.hi);
    trace!("  LO: {:016X}", cpu.lo);

    DcState::Nop
}

pub fn mfhi(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rd = ((word >> 11) & 31) as usize;
    trace!("{:08X}: MFHI {}", pc, Cpu::REG_NAMES[rd],);
    DcState::RegWrite {
        reg: rd,
        value: cpu.hi,
    }
}

pub fn mflo(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rd = ((word >> 11) & 31) as usize;
    trace!("{:08X}: MFLO {}", pc, Cpu::REG_NAMES[rd],);
    DcState::RegWrite {
        reg: rd,
        value: cpu.lo,
    }
}
