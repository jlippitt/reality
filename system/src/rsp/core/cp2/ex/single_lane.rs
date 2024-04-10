use super::{Core, Cp2, DfState};
use tracing::trace;

pub trait SingleLaneOperator {
    const NAME: &'static str;
    fn apply(cp2: &mut Cp2, input: u16) -> u16;
}

pub struct VMov;

impl SingleLaneOperator for VMov {
    const NAME: &'static str = "VMOV";

    fn apply(_cp2: &mut Cp2, input: u16) -> u16 {
        input
    }
}

pub fn single_lane<Op: SingleLaneOperator>(core: &mut Core, pc: u32, word: u32) -> DfState {
    let vt_el_raw = ((word >> 21) & 15) as usize;
    let vt = ((word >> 16) & 31) as usize;
    let vd_el_raw = ((word >> 11) & 15) as usize;
    let vd = ((word >> 6) & 31) as usize;

    trace!(
        "{:08X} {} V{:02}[E({}], V{:02}[E({}]",
        pc,
        Op::NAME,
        vd,
        vd_el_raw,
        vt,
        vt_el_raw,
    );

    let vt_el = match vt_el_raw {
        0..=1 => vd_el_raw & 0b111,
        2..=3 => (vd_el_raw & 0b110) | (vt_el_raw & 0b001),
        4..=7 => (vd_el_raw & 0b100) | (vt_el_raw & 0b011),
        _ => vt_el_raw & 0b111,
    };

    let vd_el = vd_el_raw & 0b111;

    let src = core.cp2.reg(vt);
    let mut dst = core.cp2.reg(vd);
    let acc = core.cp2.acc.as_le_array_mut();

    for (acc, value) in acc.iter_mut().zip(src.broadcast_le(vt_el).iter()) {
        *acc = (*acc & !0xffff) | (*value as u64)
    }

    dst.set_lane(vd_el, Op::apply(&mut core.cp2, src.lane(vt_el)));
    core.cp2.set_reg(vd, dst);

    DfState::Nop
}
