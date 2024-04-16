#![allow(clippy::redundant_pattern_matching)]
#![allow(clippy::upper_case_acronyms)]

use super::{Cpu, DcOperation, Float};
use std::cmp::Ordering;
use tracing::trace;

macro_rules! condition {
    ($struct:ident, $name:literal, $ordered:expr, $pattern:pat $(if $guard:expr)? $(,)?) => {
        pub struct $struct;

        impl Condition for $struct {
            const NAME: &'static str = $name;
            const ORDERED: bool = $ordered;

            fn test(ord: Option<Ordering>) -> bool {
                matches!(ord, $pattern $(if $guard)?)
            }
        }
    };
}

pub trait Condition {
    const NAME: &'static str;
    const ORDERED: bool;
    fn test(ord: Option<Ordering>) -> bool;
}

pub fn c<C: Condition, F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let ft = ((word >> 16) & 31) as usize;
    let fs = ((word >> 11) & 31) as usize;

    trace!("{:08X}: C.{}.{} F{}, F{}", pc, C::NAME, F::NAME, fs, ft);

    let result = F::cp1_reg(cpu, fs).partial_cmp(&F::cp1_reg(cpu, ft));

    if C::ORDERED && result.is_none() {
        todo!("Floating-point ordering exception");
    }

    cpu.cp1.status.set_c(C::test(result));
    trace!("  C: {}", cpu.cp1.status.c());

    DcOperation::Nop
}

condition!(F, "F", false, _ if false);
condition!(UN, "UN", false, None);
condition!(EQ, "EQ", false, Some(Ordering::Equal));
condition!(UEQ, "UEQ", false, None | Some(Ordering::Equal));
condition!(OLT, "OLT", false, Some(Ordering::Less));
condition!(ULT, "ULT", false, None | Some(Ordering::Less));
condition!(
    OLE,
    "OLE",
    false,
    Some(Ordering::Less) | Some(Ordering::Equal)
);
condition!(
    ULE,
    "ULE",
    false,
    None | Some(Ordering::Less) | Some(Ordering::Equal)
);
condition!(SF, "SF", true, _ if false);
condition!(NGLE, "NGLE", true, None);
condition!(SEQ, "EQ", true, Some(Ordering::Equal));
condition!(NGL, "NGL", true, None | Some(Ordering::Equal));
condition!(LT, "LT", true, Some(Ordering::Less));
condition!(NGE, "NGE", true, None | Some(Ordering::Less));
condition!(LE, "LE", true, Some(Ordering::Less) | Some(Ordering::Equal));
condition!(
    NGT,
    "NGT",
    true,
    None | Some(Ordering::Less) | Some(Ordering::Equal)
);
