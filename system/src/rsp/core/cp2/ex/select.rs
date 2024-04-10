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
