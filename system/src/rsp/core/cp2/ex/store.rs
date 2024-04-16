use super::{Core, Cp2, DfOperation};
use tracing::trace;

pub trait StoreOperator {
    const NAME: &'static str;
    const SHIFT: usize;
    fn apply(cp2: &Cp2, reg: usize, el: usize, addr: u32) -> DfOperation;
}

pub struct Sbv;
pub struct Ssv;
pub struct Slv;
pub struct Sdv;
pub struct Sqv;
pub struct Srv;
pub struct Spv;
pub struct Suv;
pub struct Stv;

impl StoreOperator for Sbv {
    const NAME: &'static str = "SBV";
    const SHIFT: usize = 0;

    fn apply(cp2: &Cp2, reg: usize, el: usize, addr: u32) -> DfOperation {
        DfOperation::Cp2StoreByte {
            value: cp2.reg(reg).read(el),
            addr,
        }
    }
}

impl StoreOperator for Ssv {
    const NAME: &'static str = "SSV";
    const SHIFT: usize = 1;

    fn apply(cp2: &Cp2, reg: usize, el: usize, addr: u32) -> DfOperation {
        DfOperation::Cp2StoreHalfword {
            value: cp2.reg(reg).read(el),
            addr,
        }
    }
}

impl StoreOperator for Slv {
    const NAME: &'static str = "SLV";
    const SHIFT: usize = 2;

    fn apply(cp2: &Cp2, reg: usize, el: usize, addr: u32) -> DfOperation {
        DfOperation::Cp2StoreWord {
            value: cp2.reg(reg).read(el),
            addr,
        }
    }
}

impl StoreOperator for Sdv {
    const NAME: &'static str = "SDV";
    const SHIFT: usize = 3;

    fn apply(cp2: &Cp2, reg: usize, el: usize, addr: u32) -> DfOperation {
        DfOperation::Cp2StoreDoubleword {
            value: cp2.reg(reg).read(el),
            addr,
        }
    }
}

impl StoreOperator for Sqv {
    const NAME: &'static str = "SQV";
    const SHIFT: usize = 4;

    fn apply(cp2: &Cp2, reg: usize, el: usize, addr: u32) -> DfOperation {
        DfOperation::Cp2StoreQuadword {
            vector: cp2.reg(reg),
            el,
            addr,
        }
    }
}

impl StoreOperator for Srv {
    const NAME: &'static str = "SRV";
    const SHIFT: usize = 4;

    fn apply(cp2: &Cp2, reg: usize, el: usize, addr: u32) -> DfOperation {
        DfOperation::Cp2StoreQuadwordRight {
            vector: cp2.reg(reg),
            el,
            end: addr,
        }
    }
}

impl StoreOperator for Spv {
    const NAME: &'static str = "SPV";
    const SHIFT: usize = 3;

    fn apply(cp2: &Cp2, reg: usize, el: usize, addr: u32) -> DfOperation {
        DfOperation::Cp2StorePacked {
            vector: cp2.reg(reg),
            el,
            addr,
        }
    }
}

impl StoreOperator for Suv {
    const NAME: &'static str = "SUV";
    const SHIFT: usize = 3;

    fn apply(cp2: &Cp2, reg: usize, el: usize, addr: u32) -> DfOperation {
        DfOperation::Cp2StorePackedUnsigned {
            vector: cp2.reg(reg),
            el,
            addr,
        }
    }
}

impl StoreOperator for Stv {
    const NAME: &'static str = "STV";
    const SHIFT: usize = 4;

    fn apply(_cp2: &Cp2, reg: usize, el: usize, addr: u32) -> DfOperation {
        DfOperation::Cp2StoreTranspose { reg, el, addr }
    }
}

pub fn store<Op: StoreOperator>(core: &mut Core, pc: u32, word: u32) -> DfOperation {
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

    Op::apply(
        &core.cp2,
        vt,
        el,
        core.regs[base].wrapping_add(offset) as u32,
    )
}

pub fn mfc2(core: &mut Core, pc: u32, word: u32) -> DfOperation {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;
    let el = ((word >> 7) & 15) as usize;

    trace!(
        "{:08X}: MFC2 {}, V{:02}[E{}]",
        pc,
        Core::REG_NAMES[rt],
        rd,
        el
    );

    DfOperation::RegWrite {
        reg: rt,
        value: core.cp2.reg(rd).read::<u16>(el) as i16 as i32,
    }
}

pub fn cfc2(core: &mut Core, pc: u32, word: u32) -> DfOperation {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: CFC2 {}, {}",
        pc,
        Core::REG_NAMES[rt],
        Cp2::CONTROL_REG_NAMES[rd]
    );

    DfOperation::RegWrite {
        reg: rt,
        value: core.cp2.control_reg(rd),
    }
}
