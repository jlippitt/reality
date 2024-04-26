use super::Core;
use tracing::trace;

pub fn break_(core: &mut Core) {
    trace!("{:08X}: BREAK", core.pc[0]);
    core.broke = true;
    core.opcode[1] = 0;
    core.delay[1] = false;
    core.pc[2] = core.pc[1];
}
