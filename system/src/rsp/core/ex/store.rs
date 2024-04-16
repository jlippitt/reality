use super::{Core, DfOperation};
use tracing::trace;

pub trait StoreOperator {
    const NAME: &'static str;
    fn apply(cpu: &Core, reg: usize, addr: u32) -> DfOperation;
}

pub struct Sb;
pub struct Sh;
pub struct Sw;

impl StoreOperator for Sb {
    const NAME: &'static str = "SB";

    fn apply(cpu: &Core, reg: usize, addr: u32) -> DfOperation {
        DfOperation::StoreByte {
            value: cpu.regs[reg] as u8,
            addr,
        }
    }
}

impl StoreOperator for Sh {
    const NAME: &'static str = "SH";

    fn apply(cpu: &Core, reg: usize, addr: u32) -> DfOperation {
        DfOperation::StoreHalfword {
            value: cpu.regs[reg] as u16,
            addr,
        }
    }
}

impl StoreOperator for Sw {
    const NAME: &'static str = "SW";

    fn apply(cpu: &Core, reg: usize, addr: u32) -> DfOperation {
        DfOperation::StoreWord {
            value: cpu.regs[reg] as u32,
            addr,
        }
    }
}

pub fn store<Op: StoreOperator>(cpu: &mut Core, pc: u32, word: u32) -> DfOperation {
    let base = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let offset = (word & 0xffff) as i16 as i32;

    trace!(
        "{:08X}: {} {}, {}({})",
        pc,
        Op::NAME,
        Core::REG_NAMES[rt],
        offset,
        Core::REG_NAMES[base],
    );

    Op::apply(cpu, rt, cpu.regs[base].wrapping_add(offset) as u32)
}
