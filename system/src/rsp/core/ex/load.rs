use super::{Core, DcState};
use tracing::trace;

pub trait LoadOperator {
    const NAME: &'static str;
    fn apply(reg: usize, addr: u32) -> DcState;
}

pub struct Lb;
pub struct Lbu;
pub struct Lh;
pub struct Lhu;
pub struct Lw;

impl LoadOperator for Lb {
    const NAME: &'static str = "LB";

    fn apply(reg: usize, addr: u32) -> DcState {
        DcState::LoadByte { reg, addr }
    }
}

impl LoadOperator for Lbu {
    const NAME: &'static str = "LBU";

    fn apply(reg: usize, addr: u32) -> DcState {
        DcState::LoadByteUnsigned { reg, addr }
    }
}

impl LoadOperator for Lh {
    const NAME: &'static str = "LH";

    fn apply(reg: usize, addr: u32) -> DcState {
        DcState::LoadHalfword { reg, addr }
    }
}

impl LoadOperator for Lhu {
    const NAME: &'static str = "LHU";

    fn apply(reg: usize, addr: u32) -> DcState {
        DcState::LoadHalfwordUnsigned { reg, addr }
    }
}

impl LoadOperator for Lw {
    const NAME: &'static str = "LW";

    fn apply(reg: usize, addr: u32) -> DcState {
        DcState::LoadWord { reg, addr }
    }
}

pub fn lui(_cpu: &mut Core, pc: u32, word: u32) -> DcState {
    let rt = ((word >> 16) & 31) as usize;
    let imm = (word & 0xffff) as i16;

    trace!("{:08X}: LUI {}, 0x{:04X}", pc, Core::REG_NAMES[rt], imm);

    DcState::RegWrite {
        reg: rt,
        value: ((imm as i32) << 16),
    }
}

pub fn load<Op: LoadOperator>(cpu: &mut Core, pc: u32, word: u32) -> DcState {
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

    Op::apply(rt, cpu.regs[base].wrapping_add(offset) as u32)
}
