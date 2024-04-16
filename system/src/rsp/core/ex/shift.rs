use super::{Core, DfOperation};
use tracing::trace;

pub trait ShiftOperator {
    const NAME: &'static str;
    fn apply(input: i32, amount: u32) -> i32;
}

pub struct Sll;
pub struct Srl;
pub struct Sra;

impl ShiftOperator for Sll {
    const NAME: &'static str = "SLL";

    fn apply(lhs: i32, amount: u32) -> i32 {
        (lhs as u32).wrapping_shl(amount) as i32
    }
}

impl ShiftOperator for Srl {
    const NAME: &'static str = "SRL";

    fn apply(lhs: i32, amount: u32) -> i32 {
        (lhs as u32).wrapping_shr(amount) as i32
    }
}

impl ShiftOperator for Sra {
    const NAME: &'static str = "SRA";

    fn apply(lhs: i32, amount: u32) -> i32 {
        lhs.wrapping_shr(amount & 31)
    }
}

pub fn fixed<Op: ShiftOperator>(cpu: &mut Core, pc: u32, word: u32) -> DfOperation {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;
    let sa = (word >> 6) & 31;

    trace!(
        "{:08X}: {} {}, {}, {}",
        pc,
        Op::NAME,
        Core::REG_NAMES[rd],
        Core::REG_NAMES[rt],
        sa
    );

    DfOperation::RegWrite {
        reg: rd,
        value: Op::apply(cpu.regs[rt], sa),
    }
}

pub fn variable<Op: ShiftOperator>(cpu: &mut Core, pc: u32, word: u32) -> DfOperation {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: {}V {}, {}, {}",
        pc,
        Op::NAME,
        Core::REG_NAMES[rd],
        Core::REG_NAMES[rt],
        Core::REG_NAMES[rs],
    );

    DfOperation::RegWrite {
        reg: rd,
        value: Op::apply(cpu.regs[rt], cpu.regs[rs] as u32),
    }
}
