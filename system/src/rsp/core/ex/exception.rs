use super::{Core, DfOperation};
use tracing::trace;

pub fn break_(_cpu: &mut Core, pc: u32) -> DfOperation {
    trace!("{:08X}: BREAK", pc);
    DfOperation::Break
}
