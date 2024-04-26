use super::Core;
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

pub fn fixed<Op: ShiftOperator>(core: &mut Core) {
    let rt = ((core.opcode[0] >> 16) & 31) as usize;
    let rd = ((core.opcode[0] >> 11) & 31) as usize;
    let sa = (core.opcode[0] >> 6) & 31;

    trace!(
        "{:08X}: {} {}, {}, {}",
        core.pc[0],
        Op::NAME,
        Core::REG_NAMES[rd],
        Core::REG_NAMES[rt],
        sa
    );

    core.set_reg(rd, Op::apply(core.regs[rt], sa));
}

pub fn variable<Op: ShiftOperator>(core: &mut Core) {
    let rs = ((core.opcode[0] >> 21) & 31) as usize;
    let rt = ((core.opcode[0] >> 16) & 31) as usize;
    let rd = ((core.opcode[0] >> 11) & 31) as usize;

    trace!(
        "{:08X}: {}V {}, {}, {}",
        core.pc[0],
        Op::NAME,
        Core::REG_NAMES[rd],
        Core::REG_NAMES[rt],
        Core::REG_NAMES[rs],
    );

    core.set_reg(rd, Op::apply(core.regs[rt], core.regs[rs] as u32));
}
