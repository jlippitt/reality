use super::Core;
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

pub fn i_type<Op: BitwiseOperator>(core: &mut Core) {
    let rs = ((core.opcode[0] >> 21) & 31) as usize;
    let rt = ((core.opcode[0] >> 16) & 31) as usize;
    let imm = (core.opcode[0] & 0xffff) as u64 as i32;

    trace!(
        "{:08X}: {}I {}, {}, 0x{:04X}",
        core.pc[0],
        Op::NAME,
        Core::REG_NAMES[rt],
        Core::REG_NAMES[rs],
        imm
    );

    core.set_reg(rt, Op::apply(core.regs[rs], imm));
}

pub fn r_type<Op: BitwiseOperator>(core: &mut Core) {
    let rs = ((core.opcode[0] >> 21) & 31) as usize;
    let rt = ((core.opcode[0] >> 16) & 31) as usize;
    let rd = ((core.opcode[0] >> 11) & 31) as usize;

    trace!(
        "{:08X}: {} {}, {}, {}",
        core.pc[0],
        Op::NAME,
        Core::REG_NAMES[rd],
        Core::REG_NAMES[rs],
        Core::REG_NAMES[rt],
    );

    core.set_reg(rd, Op::apply(core.regs[rs], core.regs[rt]));
}
