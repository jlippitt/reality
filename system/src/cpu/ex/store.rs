use super::{Cpu, DcState};
use tracing::trace;

pub trait StoreOperator {
    const NAME: &'static str;
    fn apply(value: i64, addr: u32) -> DcState;
}

pub struct Sb;
pub struct Sh;
pub struct Sw;

impl StoreOperator for Sb {
    const NAME: &'static str = "SB";

    fn apply(value: i64, addr: u32) -> DcState {
        DcState::StoreByte {
            value: value as u8,
            addr,
        }
    }
}

impl StoreOperator for Sh {
    const NAME: &'static str = "SH";

    fn apply(value: i64, addr: u32) -> DcState {
        DcState::StoreHalfword {
            value: value as u16,
            addr,
        }
    }
}

impl StoreOperator for Sw {
    const NAME: &'static str = "LW";

    fn apply(value: i64, addr: u32) -> DcState {
        DcState::StoreWord {
            value: value as u32,
            addr,
        }
    }
}

pub fn store<Op: StoreOperator>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let base = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let offset = (word & 0xffff) as i16 as i64;

    trace!(
        "{:08X}: {} {}, {}({})",
        pc,
        Op::NAME,
        Cpu::REG_NAMES[rt],
        offset,
        Cpu::REG_NAMES[base],
    );

    Op::apply(cpu.regs[rt], cpu.regs[base].wrapping_add(offset) as u32)
}
