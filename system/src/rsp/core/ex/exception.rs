use super::{Core, DfState};
use tracing::trace;

pub fn break_(_cpu: &mut Core, pc: u32) -> DfState {
    trace!("{:08X}: BREAK", pc);
    DfState::Break
}
