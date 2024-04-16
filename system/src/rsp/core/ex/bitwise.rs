use super::{Core, DfOperation};
use tracing::trace;

pub trait BitwiseOperator {
    const NAME: &'static str;
    fn apply(lhs: i32, rhs: i32) -> i32;
}

pub struct And;
pub struct Or;
pub struct Xor;
pub struct Nor;

impl BitwiseOperator for And {
    const NAME: &'static str = "AND";

    fn apply(lhs: i32, rhs: i32) -> i32 {
        lhs & rhs
    }
}

impl BitwiseOperator for Or {
    const NAME: &'static str = "OR";

    fn apply(lhs: i32, rhs: i32) -> i32 {
        lhs | rhs
    }
}

impl BitwiseOperator for Xor {
    const NAME: &'static str = "XOR";

    fn apply(lhs: i32, rhs: i32) -> i32 {
        lhs ^ rhs
    }
}

impl BitwiseOperator for Nor {
    const NAME: &'static str = "NOR";

    fn apply(lhs: i32, rhs: i32) -> i32 {
        !(lhs | rhs)
    }
}

pub fn i_type<Op: BitwiseOperator>(cpu: &mut Core, pc: u32, word: u32) -> DfOperation {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let imm = (word & 0xffff) as u64 as i32;

    trace!(
        "{:08X}: {}I {}, {}, 0x{:04X}",
        pc,
        Op::NAME,
        Core::REG_NAMES[rt],
        Core::REG_NAMES[rs],
        imm
    );

    DfOperation::RegWrite {
        reg: rt,
        value: Op::apply(cpu.regs[rs], imm),
    }
}

pub fn r_type<Op: BitwiseOperator>(cpu: &mut Core, pc: u32, word: u32) -> DfOperation {
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

    DfOperation::RegWrite {
        reg: rd,
        value: Op::apply(cpu.regs[rs], cpu.regs[rt]),
    }
}
