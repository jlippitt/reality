use super::{Core, Cp2, DfState};
use tracing::trace;

pub trait SingleLaneOperator {
    const NAME: &'static str;
    fn apply(cp2: &mut Cp2, input: u16) -> u16;
}

pub struct VMov;
pub struct VRcp;
pub struct VRcpl;
pub struct VRcph;
pub struct VRsq;
pub struct VRsql;
pub struct VRsqh;

impl SingleLaneOperator for VMov {
    const NAME: &'static str = "VMOV";

    fn apply(_cp2: &mut Cp2, input: u16) -> u16 {
        input
    }
}

impl SingleLaneOperator for VRcp {
    const NAME: &'static str = "VRCP";

    fn apply(cp2: &mut Cp2, input: u16) -> u16 {
        calc_reciprocal(cp2, input, input_double, value_reciprocal, 0)
    }
}

impl SingleLaneOperator for VRcpl {
    const NAME: &'static str = "VRCPL";

    fn apply(cp2: &mut Cp2, input: u16) -> u16 {
        calc_reciprocal(cp2, input, input_low, value_reciprocal, 0)
    }
}

impl SingleLaneOperator for VRcph {
    const NAME: &'static str = "VRCPH";

    fn apply(cp2: &mut Cp2, input: u16) -> u16 {
        cp2.div_in = (input as u32) << 16;
        (cp2.div_out >> 16) as u16
    }
}

impl SingleLaneOperator for VRsq {
    const NAME: &'static str = "VRSQ";

    fn apply(cp2: &mut Cp2, input: u16) -> u16 {
        calc_reciprocal(cp2, input, input_double, value_inv_sqrt, 1)
    }
}

impl SingleLaneOperator for VRsql {
    const NAME: &'static str = "VRsql";

    fn apply(cp2: &mut Cp2, input: u16) -> u16 {
        calc_reciprocal(cp2, input, input_low, value_inv_sqrt, 1)
    }
}

impl SingleLaneOperator for VRsqh {
    const NAME: &'static str = "VRsqh";

    fn apply(cp2: &mut Cp2, input: u16) -> u16 {
        cp2.div_in = (input as u32) << 16;
        (cp2.div_out >> 16) as u16
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

fn calc_reciprocal(
    cp2: &mut Cp2,
    input: u16,
    input_cb: impl Fn(&Cp2, u16) -> i32,
    value_cb: impl Fn(&Cp2, usize, usize) -> u16,
    mod_shift: u32,
) -> u16 {
    let input = input_cb(cp2, input);
    let mask = input >> 31;
    let div_in = input.wrapping_abs();

    let result = match div_in as u32 {
        0 => 0x7fff_ffff,
        0xffff_8000 => 0xffff_0000,
        _ => {
            let shift = div_in.leading_zeros();
            let index = ((div_in << shift) & 0x7fc0_0000) >> 22;
            let value = value_cb(cp2, index as usize, shift as usize);
            ((((0x10000 | value as u32 as i32) << 14) >> ((31 - shift) >> mod_shift)) ^ mask) as u32
        }
    };

    cp2.div_in = div_in as u32;
    cp2.div_out = result;

    result as u16
}

fn input_double(_cp2: &Cp2, input: u16) -> i32 {
    input as i16 as i32
}

fn input_low(cp2: &Cp2, input: u16) -> i32 {
    ((cp2.div_in & 0xffff_0000) as i32) | input as i16 as i32
}

fn value_reciprocal(cp2: &Cp2, index: usize, _shift: usize) -> u16 {
    cp2.reciprocal[index]
}

fn value_inv_sqrt(cp2: &Cp2, index: usize, shift: usize) -> u16 {
    cp2.inv_sqrt[(index & 0x1fe) | (shift & 1)]
}
