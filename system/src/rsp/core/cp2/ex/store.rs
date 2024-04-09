use super::{Core, DfState, Vector};
use tracing::trace;

pub trait StoreOperator {
    const NAME: &'static str;
    const SHIFT: usize;
    fn apply(value: Vector, el: usize, addr: u32) -> DfState;
}

pub struct Sbv;
pub struct Ssv;
pub struct Slv;
pub struct Sdv;
pub struct Sqv;

impl StoreOperator for Sbv {
    const NAME: &'static str = "SBV";
    const SHIFT: usize = 0;

    fn apply(value: Vector, el: usize, addr: u32) -> DfState {
        DfState::Cp2StoreByte {
            value: value.read(el),
            addr,
        }
    }
}

impl StoreOperator for Ssv {
    const NAME: &'static str = "LSV";
    const SHIFT: usize = 1;

    fn apply(value: Vector, el: usize, addr: u32) -> DfState {
        DfState::Cp2StoreHalfword {
            value: value.read(el),
            addr,
        }
    }
}

impl StoreOperator for Slv {
    const NAME: &'static str = "SLV";
    const SHIFT: usize = 2;

    fn apply(value: Vector, el: usize, addr: u32) -> DfState {
        DfState::Cp2StoreWord {
            value: value.read(el),
            addr,
        }
    }
}

impl StoreOperator for Sdv {
    const NAME: &'static str = "SDV";
    const SHIFT: usize = 3;

    fn apply(value: Vector, el: usize, addr: u32) -> DfState {
        DfState::Cp2StoreDoubleword {
            value: value.read(el),
            addr,
        }
    }
}

impl StoreOperator for Sqv {
    const NAME: &'static str = "SQV";
    const SHIFT: usize = 4;

    fn apply(value: Vector, el: usize, addr: u32) -> DfState {
        DfState::Cp2StoreQuadword {
            vec: value,
            el,
            addr,
        }
    }
}

pub fn store<Op: StoreOperator>(core: &mut Core, pc: u32, word: u32) -> DfState {
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

    Op::apply(
        core.cp2.reg(vt),
        el,
        core.regs[base].wrapping_add(offset) as u32,
    )
}
