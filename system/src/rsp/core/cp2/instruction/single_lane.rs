use super::{Core, Cp2};
use tracing::trace;

pub trait SingleLaneOperator {
    const NAME: &'static str;
    const VT_EL_MAGIC: bool;
    fn apply(cp2: &mut Cp2, input: u16) -> u16;
}

pub struct VMov;
pub struct VRcp;
pub struct VRcpl;
pub struct VRcph;
pub struct VRsq;
pub struct VRsql;
pub struct VRsqh;

trait ReciprocalOperator {
    const MOD_SHIFT: u32;
    fn apply(cp2: &Cp2, index: usize, _shift: usize) -> u16;
}

struct Rcp;
struct Rsq;

impl SingleLaneOperator for VMov {
    const NAME: &'static str = "VMOV";
    const VT_EL_MAGIC: bool = true;

    fn apply(_cp2: &mut Cp2, input: u16) -> u16 {
        input
    }
}

impl ReciprocalOperator for Rcp {
    const MOD_SHIFT: u32 = 0;

    fn apply(cp2: &Cp2, index: usize, _shift: usize) -> u16 {
        cp2.reciprocal[index]
    }
}

impl SingleLaneOperator for VRcp {
    const NAME: &'static str = "VRCP";
    const VT_EL_MAGIC: bool = false;

    fn apply(cp2: &mut Cp2, input: u16) -> u16 {
        let rcp_input = input as i16 as i32;
        calc_reciprocal::<Rcp>(cp2, rcp_input)
    }
}

impl SingleLaneOperator for VRcpl {
    const NAME: &'static str = "VRCPL";
    const VT_EL_MAGIC: bool = false;

    fn apply(cp2: &mut Cp2, input: u16) -> u16 {
        let rcp_input = if cp2.rcp_high {
            (cp2.rcp_in | input as u32) as i32
        } else {
            input as i16 as i32
        };

        calc_reciprocal::<Rcp>(cp2, rcp_input)
    }
}

impl SingleLaneOperator for VRcph {
    const NAME: &'static str = "VRCPH";
    const VT_EL_MAGIC: bool = false;

    fn apply(cp2: &mut Cp2, input: u16) -> u16 {
        cp2.rcp_high = true;
        cp2.rcp_in = (input as u32) << 16;
        (cp2.rcp_out >> 16) as u16
    }
}

impl ReciprocalOperator for Rsq {
    const MOD_SHIFT: u32 = 1;

    fn apply(cp2: &Cp2, index: usize, shift: usize) -> u16 {
        cp2.inv_sqrt[(index & 0x1fe) | (shift & 1)]
    }
}

impl SingleLaneOperator for VRsq {
    const NAME: &'static str = "VRSQ";
    const VT_EL_MAGIC: bool = false;

    fn apply(cp2: &mut Cp2, input: u16) -> u16 {
        let rcp_input = input as i16 as i32;
        calc_reciprocal::<Rsq>(cp2, rcp_input)
    }
}

impl SingleLaneOperator for VRsql {
    const NAME: &'static str = "VRSQL";
    const VT_EL_MAGIC: bool = false;

    fn apply(cp2: &mut Cp2, input: u16) -> u16 {
        let rcp_input = if cp2.rcp_high {
            (cp2.rcp_in | input as u32) as i32
        } else {
            input as i16 as i32
        };

        calc_reciprocal::<Rsq>(cp2, rcp_input)
    }
}

impl SingleLaneOperator for VRsqh {
    const NAME: &'static str = "VRSQH";
    const VT_EL_MAGIC: bool = false;

    fn apply(cp2: &mut Cp2, input: u16) -> u16 {
        cp2.rcp_high = true;
        cp2.rcp_in = (input as u32) << 16;
        (cp2.rcp_out >> 16) as u16
    }
}

pub fn single_lane<Op: SingleLaneOperator>(core: &mut Core) {
    let vt_el_raw = ((core.opcode[0] >> 21) & 15) as usize;
    let vt = ((core.opcode[0] >> 16) & 31) as usize;
    let vd_el_raw = ((core.opcode[0] >> 11) & 15) as usize;
    let vd = ((core.opcode[0] >> 6) & 31) as usize;

    trace!(
        "{:08X} {} V{:02}[E({}], V{:02}[E({}]",
        core.pc[0],
        Op::NAME,
        vd,
        vd_el_raw,
        vt,
        vt_el_raw,
    );

    let vt_el = if Op::VT_EL_MAGIC {
        match vt_el_raw {
            0..=1 => vd_el_raw & 0b111,
            2..=3 => (vd_el_raw & 0b110) | (vt_el_raw & 0b001),
            4..=7 => (vd_el_raw & 0b100) | (vt_el_raw & 0b011),
            _ => vt_el_raw & 0b111,
        }
    } else {
        vt_el_raw & 7
    };

    let vd_el = vd_el_raw & 0b111;

    let src = core.cp2.reg(vt);
    let mut dst = core.cp2.reg(vd);
    let acc = core.cp2.acc.as_le_array_mut();

    for (acc, value) in acc.iter_mut().zip(src.broadcast_le(vt_el_raw).iter()) {
        *acc = (*acc & !0xffff) | (*value as u64)
    }

    dst.set_lane(vd_el, Op::apply(&mut core.cp2, src.lane(vt_el)));
    core.cp2.set_reg(vd, dst);
}

pub fn vnop(core: &mut Core) {
    trace!("{:08x}: VNOP", core.pc[0]);
}

pub fn vnull(core: &mut Core) {
    trace!("{:08x}: VNULL", core.pc[0]);
}

fn calc_reciprocal<Op: ReciprocalOperator>(cp2: &mut Cp2, input: i32) -> u16 {
    let mask = input >> 31;
    let mut data = input ^ mask;

    if input > -32768 {
        data -= mask;
    }

    let result = if data == 0 {
        0x7fff_ffff
    } else if input as u32 == 0xffff_8000 {
        0xffff_0000
    } else {
        let shift = data.leading_zeros();
        let index = ((data << shift) & 0x7fc0_0000) >> 22;
        let value = Op::apply(cp2, index as usize, shift as usize);
        ((((0x10000 | value as u32 as i32) << 14) >> ((31 - shift) >> Op::MOD_SHIFT)) ^ mask) as u32
    };

    cp2.rcp_high = false;
    cp2.rcp_out = result;

    result as u16
}
