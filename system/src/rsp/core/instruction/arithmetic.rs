use super::Core;
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

pub fn i_type<Op: ArithmeticOperator, const U: bool>(core: &mut Core) {
    let rs = ((core.opcode[0] >> 21) & 31) as usize;
    let rt = ((core.opcode[0] >> 16) & 31) as usize;
    let imm = (core.opcode[0] & 0xffff) as i16 as i32;

    trace!(
        "{:08X}: {}I{} {}, {}, {}",
        core.pc[0],
        Op::NAME,
        if U { "U" } else { "" },
        Core::REG_NAMES[rt],
        Core::REG_NAMES[rs],
        imm
    );

    core.set_reg(rt, Op::apply(core.regs[rs], imm));
}

pub fn r_type<Op: ArithmeticOperator, const U: bool>(core: &mut Core) {
    let rs = ((core.opcode[0] >> 21) & 31) as usize;
    let rt = ((core.opcode[0] >> 16) & 31) as usize;
    let rd = ((core.opcode[0] >> 11) & 31) as usize;

    trace!(
        "{:08X}: {}{} {}, {}, {}",
        core.pc[0],
        Op::NAME,
        if U { "U" } else { "" },
        Core::REG_NAMES[rd],
        Core::REG_NAMES[rs],
        Core::REG_NAMES[rt],
    );

    core.set_reg(rd, Op::apply(core.regs[rs], core.regs[rt]));
}
