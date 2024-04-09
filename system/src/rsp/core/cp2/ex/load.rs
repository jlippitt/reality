use super::{Core, DfState};
use tracing::trace;

pub trait LoadOperator {
    const NAME: &'static str;
    const SHIFT: usize;
    fn apply(reg: usize, el: usize, addr: u32) -> DfState;
}

pub struct Lbv;
pub struct Lsv;
pub struct Llv;
pub struct Ldv;
pub struct Lqv;

impl LoadOperator for Lbv {
    const NAME: &'static str = "LBV";
    const SHIFT: usize = 0;

    fn apply(reg: usize, el: usize, addr: u32) -> DfState {
        DfState::Cp2LoadByte { reg, el, addr }
    }
}

impl LoadOperator for Lsv {
    const NAME: &'static str = "LSV";
    const SHIFT: usize = 1;

    fn apply(reg: usize, el: usize, addr: u32) -> DfState {
        DfState::Cp2LoadHalfword { reg, el, addr }
    }
}

impl LoadOperator for Llv {
    const NAME: &'static str = "LLV";
    const SHIFT: usize = 2;

    fn apply(reg: usize, el: usize, addr: u32) -> DfState {
        DfState::Cp2LoadWord { reg, el, addr }
    }
}

impl LoadOperator for Ldv {
    const NAME: &'static str = "LDV";
    const SHIFT: usize = 3;

    fn apply(reg: usize, el: usize, addr: u32) -> DfState {
        DfState::Cp2LoadDoubleword { reg, el, addr }
    }
}

impl LoadOperator for Lqv {
    const NAME: &'static str = "LQV";
    const SHIFT: usize = 4;

    fn apply(reg: usize, el: usize, addr: u32) -> DfState {
        DfState::Cp2LoadQuadword { reg, el, addr }
    }
}

pub fn load<Op: LoadOperator>(core: &mut Core, pc: u32, word: u32) -> DfState {
    let base = ((word >> 21) & 31) as usize;
    let vt = ((word >> 16) & 31) as usize;
    let el = ((word >> 7) & 15) as usize;
    let offset = ((word & 0x7f).wrapping_sub((word & 0x40) << 1) as i32) << Op::SHIFT;

    trace!(
        "{:08X}: {} V{:02}[e{}], {}({})",
        pc,
        Op::NAME,
        Core::REG_NAMES[vt],
        el,
        offset,
        Core::REG_NAMES[base],
    );

    Op::apply(vt, el, core.regs[base].wrapping_add(offset) as u32)
}
