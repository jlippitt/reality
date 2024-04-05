use super::{Cpu, DcState};
use tracing::trace;

pub trait BitwiseOperator {
    const NAME: &'static str;
    fn apply(lhs: i64, rhs: i64) -> i64;
}

pub struct And;
pub struct Or;
pub struct Xor;
pub struct Nor;

impl BitwiseOperator for And {
    const NAME: &'static str = "AND";

    fn apply(lhs: i64, rhs: i64) -> i64 {
        lhs & rhs
    }
}

impl BitwiseOperator for Or {
    const NAME: &'static str = "OR";

    fn apply(lhs: i64, rhs: i64) -> i64 {
        lhs | rhs
    }
}

impl BitwiseOperator for Xor {
    const NAME: &'static str = "XOR";

    fn apply(lhs: i64, rhs: i64) -> i64 {
        lhs ^ rhs
    }
}

impl BitwiseOperator for Nor {
    const NAME: &'static str = "NOR";

    fn apply(lhs: i64, rhs: i64) -> i64 {
        !(lhs | rhs)
    }
}

pub fn i_type<Op: BitwiseOperator>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let imm = (word & 0xffff) as u64 as i64;

    trace!(
        "{:08X}: {}I {}, {}, 0x{:04X}",
        pc,
        Op::NAME,
        Cpu::REG_NAMES[rt],
        Cpu::REG_NAMES[rs],
        imm
    );

    DcState::RegWrite {
        reg: rt,
        value: Op::apply(cpu.regs[rs], imm),
    }
}

pub fn r_type<Op: BitwiseOperator>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
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

    DcState::RegWrite {
        reg: rd,
        value: Op::apply(cpu.regs[rs], cpu.regs[rt]),
    }
}
