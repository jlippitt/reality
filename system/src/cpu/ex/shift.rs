use super::{Cpu, DcState};
use tracing::trace;

pub trait ShiftOperator {
    const NAME: &'static str;
    fn apply(input: i64, amount: u32) -> i64;
}

pub struct Sll;
pub struct Srl;
pub struct Sra;

impl ShiftOperator for Sll {
    const NAME: &'static str = "SLL";

    fn apply(lhs: i64, amount: u32) -> i64 {
        (lhs as u32).wrapping_shl(amount) as i32 as i64
    }
}

impl ShiftOperator for Srl {
    const NAME: &'static str = "SRL";

    fn apply(lhs: i64, amount: u32) -> i64 {
        (lhs as u32).wrapping_shr(amount) as i32 as i64
    }
}

impl ShiftOperator for Sra {
    const NAME: &'static str = "SRA";

    fn apply(lhs: i64, amount: u32) -> i64 {
        lhs.wrapping_shr(amount & 31) as i32 as i64
    }
}

pub fn fixed<Op: ShiftOperator>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;
    let sa = (word >> 6) & 31;

    trace!(
        "{:08X}: {} {}, {}, {}",
        pc,
        Op::NAME,
        Cpu::REG_NAMES[rd],
        Cpu::REG_NAMES[rt],
        sa
    );

    DcState::RegWrite {
        reg: rd,
        value: Op::apply(cpu.regs[rt], sa),
    }
}

pub fn variable<Op: ShiftOperator>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: {}V {}, {}, {}",
        pc,
        Op::NAME,
        Cpu::REG_NAMES[rd],
        Cpu::REG_NAMES[rt],
        Cpu::REG_NAMES[rs],
    );

    DcState::RegWrite {
        reg: rd,
        value: Op::apply(cpu.regs[rt], cpu.regs[rs] as u32),
    }
}
