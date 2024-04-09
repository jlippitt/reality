pub use vector::Vector;

use bitflags::bitflags;
use std::fmt::{self, Display};

mod vector;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Accumulator([u64; 8]);

bitflags! {
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
    pub struct Flags: u8 {
        const CARRY = 0x01;
        const NOT_EQUAL = 0x02;
        const COMPARE = 0x04;
        const CLIP_COMPARE = 0x08;
        const COMPARE_EXTENSION = 0x10;
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FlagVector([Flags; 8]);

impl Display for Accumulator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:012X} {:012X} {:012X} {:012X} {:012X} {:012X} {:012X} {:012X}",
            self.0[7], self.0[6], self.0[5], self.0[4], self.0[3], self.0[2], self.0[1], self.0[0],
        )
    }
}

impl Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02X}", self.bits())
    }
}

impl Display for FlagVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {} {} {}",
            self.0[7], self.0[6], self.0[5], self.0[4], self.0[3], self.0[2], self.0[1], self.0[0],
        )
    }
}
