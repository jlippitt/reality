use super::{Cpu, DcOperation};
use tracing::trace;

pub trait LoadOperator {
    const NAME: &'static str;
    fn apply(reg: usize, addr: u32) -> DcOperation;
}

pub struct Lb;
pub struct Lbu;
pub struct Lh;
pub struct Lhu;
pub struct Lw;
pub struct Lwu;
pub struct Lwl;
pub struct Lwr;
pub struct Ld;
pub struct Ldl;
pub struct Ldr;
pub struct Ll;
pub struct Lld;

impl LoadOperator for Lb {
    const NAME: &'static str = "LB";

    fn apply(reg: usize, addr: u32) -> DcOperation {
        DcOperation::LoadByte { reg, addr }
    }
}

impl LoadOperator for Lbu {
    const NAME: &'static str = "LBU";

    fn apply(reg: usize, addr: u32) -> DcOperation {
        DcOperation::LoadByteUnsigned { reg, addr }
    }
}

impl LoadOperator for Lh {
    const NAME: &'static str = "LH";

    fn apply(reg: usize, addr: u32) -> DcOperation {
        DcOperation::LoadHalfword { reg, addr }
    }
}

impl LoadOperator for Lhu {
    const NAME: &'static str = "LHU";

    fn apply(reg: usize, addr: u32) -> DcOperation {
        DcOperation::LoadHalfwordUnsigned { reg, addr }
    }
}

impl LoadOperator for Lw {
    const NAME: &'static str = "LW";

    fn apply(reg: usize, addr: u32) -> DcOperation {
        DcOperation::LoadWord { reg, addr }
    }
}

impl LoadOperator for Lwu {
    const NAME: &'static str = "LWU";

    fn apply(reg: usize, addr: u32) -> DcOperation {
        DcOperation::LoadWordUnsigned { reg, addr }
    }
}

impl LoadOperator for Lwl {
    const NAME: &'static str = "LWL";

    fn apply(reg: usize, addr: u32) -> DcOperation {
        DcOperation::LoadWordLeft { reg, addr }
    }
}

impl LoadOperator for Lwr {
    const NAME: &'static str = "LWR";

    fn apply(reg: usize, addr: u32) -> DcOperation {
        DcOperation::LoadWordRight { reg, addr }
    }
}

impl LoadOperator for Ld {
    const NAME: &'static str = "LD";

    fn apply(reg: usize, addr: u32) -> DcOperation {
        DcOperation::LoadDoubleword { reg, addr }
    }
}

impl LoadOperator for Ldl {
    const NAME: &'static str = "LDL";

    fn apply(reg: usize, addr: u32) -> DcOperation {
        DcOperation::LoadDoublewordLeft { reg, addr }
    }
}

impl LoadOperator for Ldr {
    const NAME: &'static str = "LDR";

    fn apply(reg: usize, addr: u32) -> DcOperation {
        DcOperation::LoadDoublewordRight { reg, addr }
    }
}

impl LoadOperator for Ll {
    const NAME: &'static str = "LL";

    fn apply(reg: usize, addr: u32) -> DcOperation {
        DcOperation::LoadLinked { reg, addr }
    }
}

impl LoadOperator for Lld {
    const NAME: &'static str = "LLD";

    fn apply(reg: usize, addr: u32) -> DcOperation {
        DcOperation::LoadLinkedDoubleword { reg, addr }
    }
}

pub fn lui(_cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let rt = ((word >> 16) & 31) as usize;
    let imm = (word & 0xffff) as i16;

    trace!("{:08X}: LUI {}, 0x{:04X}", pc, Cpu::REG_NAMES[rt], imm);

    DcOperation::RegWrite {
        reg: rt,
        value: ((imm as i32) << 16) as i64,
    }
}

pub fn load<Op: LoadOperator>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
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

    Op::apply(rt, cpu.regs[base].wrapping_add(offset) as u32)
}
