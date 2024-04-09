use super::{Core, DfState, Flags, Vector};
use tracing::trace;

pub trait ComputeOperator {
    const NAME: &'static str;
    fn apply(flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16;
}

pub struct VAdd;

impl ComputeOperator for VAdd {
    const NAME: &'static str = "VADD";

    fn apply(flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let carry = flags.contains(Flags::CARRY);
        let result = lhs as i16 as i32 + rhs as i16 as i32 + carry as i32;
        *acc = (*acc & !0xffff) | (result as u16 as u64);
        clamp_signed(result) as u16
    }
}

pub fn compute<Op: ComputeOperator>(core: &mut Core, pc: u32, word: u32) -> DfState {
    let el = ((word >> 21) & 15) as usize;
    let vt = ((word >> 16) & 31) as usize;
    let vs = ((word >> 11) & 31) as usize;
    let vd = ((word >> 6) & 31) as usize;

    trace!(
        "{:08X}: {} V{:02}, V{:02}, V{:02}[E{}]",
        pc,
        Op::NAME,
        vd,
        vs,
        vt,
        el
    );

    let rhs = core.cp2.reg(vt).broadcast_le(el);
    let lhs = core.cp2.reg(vs).to_le_array();
    let flags = &mut core.cp2.flags.as_le_array_mut();
    let acc = &mut core.cp2.acc.as_le_array_mut();

    let result = std::array::from_fn(|index| {
        Op::apply(&mut flags[index], &mut acc[index], lhs[index], rhs[index])
    });

    core.cp2.set_reg(vd, Vector::from_le_array(result));

    DfState::Nop
}

pub fn vsar(core: &mut Core, pc: u32, word: u32) -> DfState {
    let el = ((word >> 21) & 15) as usize;
    let vd = ((word >> 6) & 31) as usize;

    trace!("{:08X}: VSAR V{:02}, V{:02}[E{}]", pc, vd, vd, el);

    if (8..=10).contains(&el) {
        let shift = 32 - ((el - 8) << 4);
        let acc = core.cp2.acc.as_le_array();
        let result = std::array::from_fn(|index| (acc[index] >> shift) as u16);
        core.cp2.set_reg(vd, Vector::from_le_array(result));
    } else {
        core.cp2.set_reg(vd, Vector::default());
    }

    DfState::Nop
}

fn clamp_signed(value: i32) -> i16 {
    value.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}
