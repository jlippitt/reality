use super::Cpu;
use tracing::trace;

pub trait MulDivOperator {
    const NAME: &'static str;
    const STALL: u64;
    fn apply(lhs: i64, rhs: i64) -> (i64, i64);
}

pub struct Mult;
pub struct Dmult;
pub struct Multu;
pub struct Dmultu;
pub struct Div;
pub struct Divu;
pub struct Ddiv;
pub struct Ddivu;

impl MulDivOperator for Mult {
    const NAME: &'static str = "MULT";
    const STALL: u64 = 5;

    fn apply(lhs: i64, rhs: i64) -> (i64, i64) {
        let result = lhs as i32 as i64 * rhs as i32 as i64;
        ((result >> 32), result as i32 as i64)
    }
}

impl MulDivOperator for Dmult {
    const NAME: &'static str = "DMULT";
    const STALL: u64 = 8;

    fn apply(lhs: i64, rhs: i64) -> (i64, i64) {
        let result = lhs as i128 * rhs as i128;
        ((result >> 64) as i64, result as i64)
    }
}

impl MulDivOperator for Multu {
    const NAME: &'static str = "MULTU";
    const STALL: u64 = 5;

    fn apply(lhs: i64, rhs: i64) -> (i64, i64) {
        let result = lhs as u32 as u64 * rhs as u32 as u64;
        (((result as i64) >> 32), result as i32 as i64)
    }
}

impl MulDivOperator for Dmultu {
    const NAME: &'static str = "DMULTU";
    const STALL: u64 = 8;

    fn apply(lhs: i64, rhs: i64) -> (i64, i64) {
        let result = lhs as u64 as u128 * rhs as u64 as u128;
        ((result >> 64) as i64, result as i64)
    }
}

impl MulDivOperator for Div {
    const NAME: &'static str = "DIV";
    const STALL: u64 = 37;

    fn apply(lhs: i64, rhs: i64) -> (i64, i64) {
        if rhs != 0 {
            (
                (lhs as i32).wrapping_rem(rhs as i32) as i64,
                (lhs as i32).wrapping_div(rhs as i32) as i64,
            )
        } else {
            (
                lhs as i32 as i64,
                if lhs < 0 { 1 } else { u32::MAX as i32 as i64 },
            )
        }
    }
}

impl MulDivOperator for Ddiv {
    const NAME: &'static str = "DDIV";
    const STALL: u64 = 69;

    fn apply(lhs: i64, rhs: i64) -> (i64, i64) {
        if rhs != 0 {
            (lhs.wrapping_rem(rhs), lhs.wrapping_div(rhs))
        } else {
            (lhs, if lhs < 0 { 1 } else { u64::MAX as i32 as i64 })
        }
    }
}

impl MulDivOperator for Divu {
    const NAME: &'static str = "DIVU";
    const STALL: u64 = 37;

    fn apply(lhs: i64, rhs: i64) -> (i64, i64) {
        if rhs != 0 {
            (
                (lhs as u32 % rhs as u32) as i64,
                (lhs as u32 / rhs as u32) as i64,
            )
        } else {
            (lhs as i32 as i64, u32::MAX as i32 as i64)
        }
    }
}

impl MulDivOperator for Ddivu {
    const NAME: &'static str = "DDIVU";
    const STALL: u64 = 69;

    fn apply(lhs: i64, rhs: i64) -> (i64, i64) {
        if rhs != 0 {
            (
                (lhs as u64 % rhs as u64) as i64,
                (lhs as u64 / rhs as u64) as i64,
            )
        } else {
            (lhs, u64::MAX as i64)
        }
    }
}

pub fn mul_div<Op: MulDivOperator>(cpu: &mut Cpu) {
    let rs = ((cpu.opcode[0] >> 21) & 31) as usize;
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;

    trace!(
        "{:08X}: {} {}, {}",
        cpu.pc[0],
        Op::NAME,
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
    );

    (cpu.hi, cpu.lo) = Op::apply(cpu.regs[rs], cpu.regs[rt]);

    trace!("  HI: {:016X}", cpu.hi);
    trace!("  LO: {:016X}", cpu.lo);

    cpu.stall += Op::STALL;
}

pub fn mfhi(cpu: &mut Cpu) {
    let rd = ((cpu.opcode[0] >> 11) & 31) as usize;
    trace!("{:08X}: MFHI {}", cpu.pc[0], Cpu::REG_NAMES[rd],);
    cpu.set_reg(rd, cpu.hi);
}

pub fn mflo(cpu: &mut Cpu) {
    let rd = ((cpu.opcode[0] >> 11) & 31) as usize;
    trace!("{:08X}: MFLO {}", cpu.pc[0], Cpu::REG_NAMES[rd],);
    cpu.set_reg(rd, cpu.lo);
}

pub fn mthi(cpu: &mut Cpu) {
    let rs = ((cpu.opcode[0] >> 21) & 31) as usize;
    trace!("{:08X}: MTHI {}", cpu.pc[0], Cpu::REG_NAMES[rs],);
    cpu.hi = cpu.regs[rs];
    trace!("  HI: {:016X}", cpu.hi);
}

pub fn mtlo(cpu: &mut Cpu) {
    let rs = ((cpu.opcode[0] >> 21) & 31) as usize;
    trace!("{:08X}: MTLO {}", cpu.pc[0], Cpu::REG_NAMES[rs],);
    cpu.lo = cpu.regs[rs];
    trace!("  LO: {:016X}", cpu.lo);
}
