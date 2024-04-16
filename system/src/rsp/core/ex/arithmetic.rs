use super::{Core, DfOperation};
use tracing::trace;

pub trait ArithmeticOperator {
    const NAME: &'static str;
    fn apply_checked(lhs: i32, rhs: i32) -> Option<i32>;
    fn apply_unchecked(lhs: i32, rhs: i32) -> i32;
}

pub struct Add;
pub struct Sub;

impl ArithmeticOperator for Add {
    const NAME: &'static str = "ADD";

    fn apply_checked(lhs: i32, rhs: i32) -> Option<i32> {
        lhs.checked_add(rhs)
    }

    fn apply_unchecked(lhs: i32, rhs: i32) -> i32 {
        lhs.wrapping_add(rhs)
    }
}

impl ArithmeticOperator for Sub {
    const NAME: &'static str = "SUB";

    fn apply_checked(lhs: i32, rhs: i32) -> Option<i32> {
        lhs.checked_sub(rhs)
    }

    fn apply_unchecked(lhs: i32, rhs: i32) -> i32 {
        lhs.wrapping_sub(rhs)
    }
}

pub fn i_type_checked<Op: ArithmeticOperator>(cpu: &mut Core, pc: u32, word: u32) -> DfOperation {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let imm = (word & 0xffff) as i16 as i32;

    trace!(
        "{:08X}: {}I {}, {}, {}",
        pc,
        Op::NAME,
        Core::REG_NAMES[rt],
        Core::REG_NAMES[rs],
        imm
    );

    let Some(result) = Op::apply_checked(cpu.regs[rs], imm) else {
        todo!("Overflow exception");
    };

    DfOperation::RegWrite {
        reg: rt,
        value: result,
    }
}

pub fn i_type_unchecked<Op: ArithmeticOperator>(cpu: &mut Core, pc: u32, word: u32) -> DfOperation {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let imm = (word & 0xffff) as i16 as i32;

    trace!(
        "{:08X}: {}IU {}, {}, {}",
        pc,
        Op::NAME,
        Core::REG_NAMES[rt],
        Core::REG_NAMES[rs],
        imm
    );

    DfOperation::RegWrite {
        reg: rt,
        value: Op::apply_unchecked(cpu.regs[rs], imm),
    }
}

pub fn r_type_checked<Op: ArithmeticOperator>(cpu: &mut Core, pc: u32, word: u32) -> DfOperation {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: {} {}, {}, {}",
        pc,
        Op::NAME,
        Core::REG_NAMES[rd],
        Core::REG_NAMES[rs],
        Core::REG_NAMES[rt],
    );

    let Some(result) = Op::apply_checked(cpu.regs[rs], cpu.regs[rt]) else {
        todo!("Overflow exception");
    };

    DfOperation::RegWrite {
        reg: rd,
        value: result,
    }
}

pub fn r_type_unchecked<Op: ArithmeticOperator>(cpu: &mut Core, pc: u32, word: u32) -> DfOperation {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: {}U {}, {}, {}",
        pc,
        Op::NAME,
        Core::REG_NAMES[rd],
        Core::REG_NAMES[rs],
        Core::REG_NAMES[rt],
    );

    DfOperation::RegWrite {
        reg: rd,
        value: Op::apply_unchecked(cpu.regs[rs], cpu.regs[rt]),
    }
}
