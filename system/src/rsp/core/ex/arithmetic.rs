use super::{Core, DfOperation};
use tracing::trace;

pub trait ArithmeticOperator {
    const NAME: &'static str;
    fn apply(lhs: i32, rhs: i32) -> i32;
}

pub struct Add;
pub struct Sub;

impl ArithmeticOperator for Add {
    const NAME: &'static str = "ADD";

    fn apply(lhs: i32, rhs: i32) -> i32 {
        lhs.wrapping_add(rhs)
    }
}

impl ArithmeticOperator for Sub {
    const NAME: &'static str = "SUB";

    fn apply(lhs: i32, rhs: i32) -> i32 {
        lhs.wrapping_sub(rhs)
    }
}

pub fn i_type<Op: ArithmeticOperator, const U: bool>(
    cpu: &mut Core,
    pc: u32,
    word: u32,
) -> DfOperation {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let imm = (word & 0xffff) as i16 as i32;

    trace!(
        "{:08X}: {}I{} {}, {}, {}",
        pc,
        Op::NAME,
        if U { "U" } else { "" },
        Core::REG_NAMES[rt],
        Core::REG_NAMES[rs],
        imm
    );

    DfOperation::RegWrite {
        reg: rt,
        value: Op::apply(cpu.regs[rs], imm),
    }
}

pub fn r_type<Op: ArithmeticOperator, const U: bool>(
    cpu: &mut Core,
    pc: u32,
    word: u32,
) -> DfOperation {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: {}{} {}, {}, {}",
        pc,
        Op::NAME,
        if U { "U" } else { "" },
        Core::REG_NAMES[rd],
        Core::REG_NAMES[rs],
        Core::REG_NAMES[rt],
    );

    DfOperation::RegWrite {
        reg: rd,
        value: Op::apply(cpu.regs[rs], cpu.regs[rt]),
    }
}
