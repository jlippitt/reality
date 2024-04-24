use super::compute::ComputeOperator;
use super::Flags;
use std::marker::PhantomData;

pub trait SelectOperator {
    const NAME: &'static str;
    fn apply(flags: &mut Flags, lhs: u16, rhs: u16) -> bool;
}

pub struct Select<Op: SelectOperator> {
    _phantom: PhantomData<Op>,
}

pub type VEq = Select<Eq>;
pub type VNe = Select<Ne>;
pub type VGe = Select<Ge>;
pub type VLt = Select<Lt>;
pub struct VCl;
pub struct VCh;
pub struct VCr;
pub struct VMrg;

pub struct Eq;
pub struct Ne;
pub struct Ge;
pub struct Lt;

impl<Op: SelectOperator> ComputeOperator for Select<Op> {
    const NAME: &'static str = Op::NAME;

    fn apply(flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let condition = Op::apply(flags, lhs, rhs);
        let result = if condition { lhs } else { rhs };
        *acc = (*acc & !0xffff) | result as u64;
        flags.set(Flags::COMPARE, condition);
        flags.remove(Flags::CARRY | Flags::NOT_EQUAL | Flags::CLIP_COMPARE);
        result
    }
}

impl SelectOperator for Eq {
    const NAME: &'static str = "VEQ";

    fn apply(flags: &mut Flags, lhs: u16, rhs: u16) -> bool {
        lhs == rhs && !flags.contains(Flags::NOT_EQUAL)
    }
}

impl SelectOperator for Ne {
    const NAME: &'static str = "VNE";

    fn apply(flags: &mut Flags, lhs: u16, rhs: u16) -> bool {
        lhs != rhs || flags.contains(Flags::NOT_EQUAL)
    }
}

impl SelectOperator for Ge {
    const NAME: &'static str = "VGE";

    fn apply(flags: &mut Flags, lhs: u16, rhs: u16) -> bool {
        (lhs as i16) > (rhs as i16)
            || (lhs == rhs && !flags.contains(Flags::CARRY | Flags::NOT_EQUAL))
    }
}

impl SelectOperator for Lt {
    const NAME: &'static str = "VLT";

    fn apply(flags: &mut Flags, lhs: u16, rhs: u16) -> bool {
        (lhs as i16) < (rhs as i16)
            || (lhs == rhs && flags.contains(Flags::CARRY | Flags::NOT_EQUAL))
    }
}

impl ComputeOperator for VCl {
    const NAME: &'static str = "VCL";

    fn apply(flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let result = if flags.contains(Flags::CARRY) {
            let lt = if flags.contains(Flags::NOT_EQUAL) {
                flags.contains(Flags::COMPARE)
            } else {
                let result = lhs as u32 + rhs as u32;

                let lt = if flags.contains(Flags::COMPARE_EXTENSION) {
                    result <= 0x10000
                } else {
                    result == 0
                };

                flags.set(Flags::COMPARE, lt);
                lt
            };

            if lt {
                (rhs as i16).wrapping_neg() as u16
            } else {
                lhs
            }
        } else {
            let ge = if flags.contains(Flags::NOT_EQUAL) {
                flags.contains(Flags::CLIP_COMPARE)
            } else {
                let ge = lhs >= rhs;
                flags.set(Flags::CLIP_COMPARE, ge);
                ge
            };

            if ge {
                rhs
            } else {
                lhs
            }
        };

        *acc = (*acc & !0xffff) | result as u64;
        flags.remove(Flags::CARRY | Flags::NOT_EQUAL | Flags::COMPARE_EXTENSION);
        result
    }
}

impl ComputeOperator for VCh {
    const NAME: &'static str = "VCH";

    fn apply(flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let carry = (lhs as i16 ^ rhs as i16) < 0;
        flags.set(Flags::CARRY, carry);

        let result = if carry {
            let value = (lhs as i16).wrapping_add(rhs as i16);
            let lt = value <= 0;

            flags.set(Flags::NOT_EQUAL, value != 0 && (lhs != !rhs));
            flags.set(Flags::COMPARE, lt);
            flags.set(Flags::CLIP_COMPARE, (rhs as i16) < 0);
            flags.set(Flags::COMPARE_EXTENSION, value == -1);

            if lt {
                (rhs as i16).wrapping_neg() as u16
            } else {
                lhs
            }
        } else {
            let value = (lhs as i16).wrapping_sub(rhs as i16);
            let ge = value >= 0;

            flags.set(Flags::NOT_EQUAL, value != 0 && (lhs != !rhs));
            flags.set(Flags::COMPARE, (rhs as i16) < 0);
            flags.set(Flags::CLIP_COMPARE, ge);
            flags.remove(Flags::COMPARE_EXTENSION);

            if ge {
                rhs
            } else {
                lhs
            }
        };

        *acc = (*acc & !0xffff) | result as u64;
        result
    }
}

impl ComputeOperator for VCr {
    const NAME: &'static str = "VCR";

    fn apply(flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let result = if (lhs as i16 ^ rhs as i16) < 0 {
            flags.set(Flags::CLIP_COMPARE, (rhs as i16) < 0);
            let lt = (lhs as i16).wrapping_add(rhs as i16) < 0;
            flags.set(Flags::COMPARE, lt);

            if lt {
                !rhs
            } else {
                lhs
            }
        } else {
            flags.set(Flags::COMPARE, (rhs as i16) < 0);

            let ge = (lhs as i16).wrapping_sub(rhs as i16) >= 0;
            flags.set(Flags::CLIP_COMPARE, ge);

            if ge {
                rhs
            } else {
                lhs
            }
        };

        *acc = (*acc & !0xffff) | result as u64;
        result
    }
}

impl ComputeOperator for VMrg {
    const NAME: &'static str = "VMRG";

    fn apply(flags: &mut Flags, acc: &mut u64, lhs: u16, rhs: u16) -> u16 {
        let result = if flags.contains(Flags::COMPARE) {
            lhs
        } else {
            rhs
        };

        *acc = (*acc & !0xffff) | result as u64;
        flags.remove(Flags::CARRY | Flags::NOT_EQUAL);
        result
    }
}
