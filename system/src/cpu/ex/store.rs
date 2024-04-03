use super::{Cpu, DcState};
use tracing::trace;

pub trait StoreOperator {
    const NAME: &'static str;
    fn apply(value: i64, addr: u32) -> DcState;
}

pub struct Sb;
pub struct Sh;
pub struct Sw;
pub struct Swl;
pub struct Swr;
pub struct Sd;
pub struct Sdl;
pub struct Sdr;

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
    const NAME: &'static str = "SW";

    fn apply(value: i64, addr: u32) -> DcState {
        DcState::StoreWord {
            value: value as u32,
            addr,
        }
    }
}

impl StoreOperator for Swl {
    const NAME: &'static str = "SWL";

    fn apply(value: i64, addr: u32) -> DcState {
        DcState::StoreWordLeft {
            value: value as u32,
            addr,
        }
    }
}

impl StoreOperator for Swr {
    const NAME: &'static str = "SWR";

    fn apply(value: i64, addr: u32) -> DcState {
        DcState::StoreWordRight {
            value: value as u32,
            addr,
        }
    }
}

impl StoreOperator for Sd {
    const NAME: &'static str = "SD";

    fn apply(value: i64, addr: u32) -> DcState {
        DcState::StoreDoubleword {
            value: value as u64,
            addr,
        }
    }
}

impl StoreOperator for Sdl {
    const NAME: &'static str = "SDL";

    fn apply(value: i64, addr: u32) -> DcState {
        DcState::StoreDoublewordLeft {
            value: value as u64,
            addr,
        }
    }
}

impl StoreOperator for Sdr {
    const NAME: &'static str = "SDR";

    fn apply(value: i64, addr: u32) -> DcState {
        DcState::StoreDoublewordRight {
            value: value as u64,
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
