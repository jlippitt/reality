use super::{Core, DfState, Flags, Vector};
use tracing::trace;

pub trait ComputeOperator {
    const NAME: &'static str;
    fn apply(flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16;
}

pub struct VMulf;
pub struct VMulu;
pub struct VMudl;
pub struct VMudm;
pub struct VMudn;
pub struct VMudh;
pub struct VMadl;
pub struct VMacf;
pub struct VMacu;
pub struct VMadm;
pub struct VMadn;
pub struct VMadh;
pub struct VAdd;
pub struct VAddc;
pub struct VSub;
pub struct VSubc;

impl ComputeOperator for VMulf {
    const NAME: &'static str = "VMULF";

    fn apply(_flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let result = (lhs as i16 as i64 * rhs as i16 as i64) << 1;
        *acc = (0x8000 + result) as u64 & 0xffff_ffff_ffff;
        clamp_accumulator_high(*acc)
    }
}

impl ComputeOperator for VMulu {
    const NAME: &'static str = "VMULU";

    fn apply(_flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let result = (lhs as i16 as i64 * rhs as i16 as i64) << 1;
        *acc = (0x8000 + result) as u64 & 0xffff_ffff_ffff;

        if ((*acc >> 32) as i16) < 0 {
            return 0;
        }

        if ((*acc >> 32) as i16) ^ ((*acc >> 16) as i16) < 0 {
            return u16::MAX;
        }

        (*acc >> 16) as u16
    }
}

impl ComputeOperator for VMudl {
    const NAME: &'static str = "VMUDL";

    fn apply(_flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let result = ((lhs as u32).wrapping_mul(rhs as u32) >> 16) as i32 as i64;
        *acc = result as u64;
        *acc as u16
    }
}

impl ComputeOperator for VMudm {
    const NAME: &'static str = "VMUDM";

    fn apply(_flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let result = (lhs as i16 as u32).wrapping_mul(rhs as u32) as i32 as i64;
        *acc = result as u64;
        (*acc as i64 >> 16) as u16
    }
}

impl ComputeOperator for VMudn {
    const NAME: &'static str = "VMUDN";

    fn apply(_flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let result = (lhs as u32).wrapping_mul(rhs as i16 as u32) as i32 as i64;
        *acc = result as u64;
        *acc as u16
    }
}

impl ComputeOperator for VMudh {
    const NAME: &'static str = "VMUDH";

    fn apply(_flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let result = ((lhs as i16 as i32).wrapping_mul(rhs as i16 as i32) as i64) << 16;
        *acc = result as u64;
        clamp_accumulator_high(*acc)
    }
}

impl ComputeOperator for VMacf {
    const NAME: &'static str = "VMACF";

    fn apply(_flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let result = (lhs as i16 as i64 * rhs as i16 as i64) << 1;
        *acc = (*acc as i64 + result) as u64 & 0xffff_ffff_ffff;
        clamp_accumulator_high(*acc)
    }
}

impl ComputeOperator for VMacu {
    const NAME: &'static str = "VMACU";

    fn apply(_flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let result = (lhs as i16 as i64 * rhs as i16 as i64) << 1;
        *acc = (*acc as i64 + result) as u64 & 0xffff_ffff_ffff;

        if ((*acc >> 32) as i16) < 0 {
            return 0;
        }

        if ((*acc >> 32) as i16) ^ ((*acc >> 16) as i16) < 0 {
            return u16::MAX;
        }

        (*acc >> 16) as u16
    }
}

impl ComputeOperator for VMadl {
    const NAME: &'static str = "VMADL";

    fn apply(_flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let result = ((lhs as u32).wrapping_mul(rhs as u32) >> 16) as i64;
        *acc = (*acc as i64 + result) as u64 & 0xffff_ffff_ffff;
        clamp_accumulator_low(*acc)
    }
}

impl ComputeOperator for VMadm {
    const NAME: &'static str = "VMADM";

    fn apply(_flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let result = (lhs as i16 as u32).wrapping_mul(rhs as u32) as i32 as i64;
        *acc = (*acc as i64 + result) as u64 & 0xffff_ffff_ffff;
        clamp_accumulator_high(*acc)
    }
}

impl ComputeOperator for VMadn {
    const NAME: &'static str = "VMADN";

    fn apply(_flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let result = lhs as u64 as i64 * rhs as i16 as i64;
        *acc = (*acc as i64 + result) as u64 & 0xffff_ffff_ffff;
        clamp_accumulator_low(*acc)
    }
}

impl ComputeOperator for VMadh {
    const NAME: &'static str = "VMADH";

    fn apply(_flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let result = ((lhs as i16 as i32).wrapping_mul(rhs as i16 as i32) as i64) << 16;
        *acc = (*acc as i64 + result) as u64 & 0xffff_ffff_ffff;
        clamp_accumulator_high(*acc)
    }
}

impl ComputeOperator for VAdd {
    const NAME: &'static str = "VADD";

    fn apply(flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let carry = flags.contains(Flags::CARRY);
        let result = lhs as i16 as i32 + rhs as i16 as i32 + carry as i32;
        *acc = (*acc & !0xffff) | (result as u16 as u64);
        flags.remove(Flags::CARRY | Flags::NOT_EQUAL);
        clamp_signed(result) as u16
    }
}

impl ComputeOperator for VAddc {
    const NAME: &'static str = "VADDC";

    fn apply(flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let result = lhs as u32 + rhs as u32;
        *acc = (*acc & !0xffff) | (result as u16 as u64);
        flags.set(Flags::CARRY, (result & 0x0001_0000) != 0);
        flags.remove(Flags::NOT_EQUAL);
        result as u16
    }
}

impl ComputeOperator for VSub {
    const NAME: &'static str = "VSUB";

    fn apply(flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let carry = flags.contains(Flags::CARRY);
        let result = lhs as i16 as i32 - rhs as i16 as i32 - carry as i32;
        *acc = (*acc & !0xffff) | (result as u16 as u64);
        flags.remove(Flags::CARRY | Flags::NOT_EQUAL);
        clamp_signed(result) as u16
    }
}

impl ComputeOperator for VSubc {
    const NAME: &'static str = "VSUBC";

    fn apply(flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let result = lhs as i32 - rhs as i32;
        *acc = (*acc & !0xffff) | (result as u16 as u64);
        flags.set(Flags::CARRY, result < 0);
        flags.set(Flags::NOT_EQUAL, result != 0);
        result as u16
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

fn clamp_accumulator_high(value: u64) -> u16 {
    if ((value >> 32) as i16) < 0 {
        if (value >> 32) as u16 != 0xffff || ((value >> 16) as i16) >= 0 {
            return 0x8000;
        }
    } else if (((value >> 32) as u16) != 0) || ((value >> 16) as i16) < 0 {
        return 0x7fff;
    }

    (value >> 16) as u16
}

fn clamp_accumulator_low(value: u64) -> u16 {
    if ((value >> 32) as i16) < 0 {
        if (value >> 32) as u16 != 0xffff || ((value >> 16) as i16) >= 0 {
            return 0;
        }
    } else if (((value >> 32) as u16) != 0) || ((value >> 16) as i16) < 0 {
        return 0xffff;
    }

    value as u16
}
