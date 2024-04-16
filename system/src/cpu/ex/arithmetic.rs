use super::{Cpu, DcOperation};
use tracing::trace;

pub trait ArithmeticOperator {
    const NAME: &'static str;
    fn apply_checked(lhs: i64, rhs: i64) -> Option<i64>;
    fn apply_unchecked(lhs: i64, rhs: i64) -> i64;
}

pub struct Add;
pub struct Dadd;
pub struct Sub;
pub struct Dsub;

impl ArithmeticOperator for Add {
    const NAME: &'static str = "ADD";

    fn apply_checked(lhs: i64, rhs: i64) -> Option<i64> {
        (lhs as i32)
            .checked_add(rhs as i32)
            .map(|result| result as i64)
    }

    fn apply_unchecked(lhs: i64, rhs: i64) -> i64 {
        (lhs as i32).wrapping_add(rhs as i32) as i64
    }
}

impl ArithmeticOperator for Dadd {
    const NAME: &'static str = "DADD";

    fn apply_checked(lhs: i64, rhs: i64) -> Option<i64> {
        lhs.checked_add(rhs)
    }

    fn apply_unchecked(lhs: i64, rhs: i64) -> i64 {
        lhs.wrapping_add(rhs)
    }
}

impl ArithmeticOperator for Sub {
    const NAME: &'static str = "SUB";

    fn apply_checked(lhs: i64, rhs: i64) -> Option<i64> {
        (lhs as i32)
            .checked_sub(rhs as i32)
            .map(|result| result as i64)
    }

    fn apply_unchecked(lhs: i64, rhs: i64) -> i64 {
        (lhs as i32).wrapping_sub(rhs as i32) as i64
    }
}

impl ArithmeticOperator for Dsub {
    const NAME: &'static str = "DSUB";

    fn apply_checked(lhs: i64, rhs: i64) -> Option<i64> {
        lhs.checked_sub(rhs)
    }

    fn apply_unchecked(lhs: i64, rhs: i64) -> i64 {
        lhs.wrapping_sub(rhs)
    }
}

pub fn i_type_checked<Op: ArithmeticOperator>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let imm = (word & 0xffff) as i16 as i64;

    trace!(
        "{:08X}: {}I {}, {}, {}",
        pc,
        Op::NAME,
        Cpu::REG_NAMES[rt],
        Cpu::REG_NAMES[rs],
        imm
    );

    let Some(result) = Op::apply_checked(cpu.regs[rs], imm) else {
        todo!("Overflow exception");
    };

    DcOperation::RegWrite {
        reg: rt,
        value: result,
    }
}

pub fn i_type_unchecked<Op: ArithmeticOperator>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let imm = (word & 0xffff) as i16 as i64;

    trace!(
        "{:08X}: {}IU {}, {}, {}",
        pc,
        Op::NAME,
        Cpu::REG_NAMES[rt],
        Cpu::REG_NAMES[rs],
        imm
    );

    DcOperation::RegWrite {
        reg: rt,
        value: Op::apply_unchecked(cpu.regs[rs], imm),
    }
}

pub fn r_type_checked<Op: ArithmeticOperator>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: {} {}, {}, {}",
        pc,
        Op::NAME,
        Cpu::REG_NAMES[rd],
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
    );

    let Some(result) = Op::apply_checked(cpu.regs[rs], cpu.regs[rt]) else {
        todo!("Overflow exception");
    };

    DcOperation::RegWrite {
        reg: rd,
        value: result,
    }
}

pub fn r_type_unchecked<Op: ArithmeticOperator>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: {}U {}, {}, {}",
        pc,
        Op::NAME,
        Cpu::REG_NAMES[rd],
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
    );

    DcOperation::RegWrite {
        reg: rd,
        value: Op::apply_unchecked(cpu.regs[rs], cpu.regs[rt]),
    }
}
