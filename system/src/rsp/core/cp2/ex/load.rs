use super::{Core, Cp2, DfState};
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
pub struct Lrv;

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

impl LoadOperator for Lrv {
    const NAME: &'static str = "LRV";
    const SHIFT: usize = 4;

    fn apply(reg: usize, el: usize, addr: u32) -> DfState {
        DfState::Cp2LoadQuadwordRight { reg, el, end: addr }
    }
}

pub fn load<Op: LoadOperator>(core: &mut Core, pc: u32, word: u32) -> DfState {
    let base = ((word >> 21) & 31) as usize;
    let vt = ((word >> 16) & 31) as usize;
    let el = ((word >> 7) & 15) as usize;
    let offset = ((word & 0x7f).wrapping_sub((word & 0x40) << 1) as i32) << Op::SHIFT;

    trace!(
        "{:08X}: {} V{:02}[E{}], {}({})",
        pc,
        Op::NAME,
        vt,
        el,
        offset,
        Core::REG_NAMES[base],
    );

    Op::apply(vt, el, core.regs[base].wrapping_add(offset) as u32)
}

pub fn mtc2(core: &mut Core, pc: u32, word: u32) -> DfState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;
    let el = ((word >> 7) & 15) as usize;

    trace!(
        "{:08X}: MTC2 {}, V{:02}[E{}]",
        pc,
        Core::REG_NAMES[rt],
        rd,
        el
    );

    let mut vector = core.cp2.reg(rd);
    vector.write(el, core.regs[rt] as u16);
    core.cp2.set_reg(rd, vector);

    DfState::Nop
}

pub fn ctc2(core: &mut Core, pc: u32, word: u32) -> DfState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: CFC2 {}, {}",
        pc,
        Core::REG_NAMES[rt],
        Cp2::CONTROL_REG_NAMES[rd]
    );

    core.cp2.set_control_reg(rd, core.regs[rt]);

    DfState::Nop
}
