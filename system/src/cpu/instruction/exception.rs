use super::cp0;
use super::Cpu;
use tracing::trace;

pub trait TrapOperator {
    const NAME: &'static str;
    fn apply(lhs: i64, rhs: i64) -> bool;
}

pub struct Tge;
pub struct Tgeu;
pub struct Tlt;
pub struct Tltu;
pub struct Teq;
pub struct Tne;

impl TrapOperator for Tge {
    const NAME: &'static str = "TGE";

    fn apply(lhs: i64, rhs: i64) -> bool {
        lhs >= rhs
    }
}

impl TrapOperator for Tgeu {
    const NAME: &'static str = "TGEU";

    fn apply(lhs: i64, rhs: i64) -> bool {
        (lhs as u64) >= (rhs as u64)
    }
}

impl TrapOperator for Tlt {
    const NAME: &'static str = "TLT";

    fn apply(lhs: i64, rhs: i64) -> bool {
        lhs < rhs
    }
}

impl TrapOperator for Tltu {
    const NAME: &'static str = "TLTU";

    fn apply(lhs: i64, rhs: i64) -> bool {
        (lhs as u64) < (rhs as u64)
    }
}

impl TrapOperator for Teq {
    const NAME: &'static str = "TEQ";

    fn apply(lhs: i64, rhs: i64) -> bool {
        lhs == rhs
    }
}

impl TrapOperator for Tne {
    const NAME: &'static str = "TNE";

    fn apply(lhs: i64, rhs: i64) -> bool {
        lhs != rhs
    }
}

pub fn cop2(cpu: &mut Cpu) {
    if cpu.cp0.cp2_usable() {
        match (cpu.opcode[0] >> 21) & 31 {
            0o00 | 0x01 | 0x02 | 0x04 | 0x05 | 0x06 => (),
            _ => cp0::except(cpu, cp0::Exception::ReservedInstruction(2)),
        }
    } else {
        cp0::except(cpu, cp0::Exception::CoprocessorUnusable(2));
    }
}

pub fn syscall(cpu: &mut Cpu) {
    trace!("{:08X}: SYSCALL", cpu.pc[0]);
    cp0::except(cpu, cp0::Exception::Syscall);
}

pub fn break_(cpu: &mut Cpu) {
    trace!("{:08X}: BREAK", cpu.pc[0]);
    cp0::except(cpu, cp0::Exception::Breakpoint);
}

pub fn trap_i_type<Op: TrapOperator>(cpu: &mut Cpu) {
    let rs = ((cpu.opcode[0] >> 21) & 31) as usize;
    let imm = (cpu.opcode[0] & 0xffff) as i16 as i64;

    trace!(
        "{:08X}: {}I {}, {}",
        cpu.pc[0],
        Op::NAME,
        Cpu::REG_NAMES[rs],
        imm,
    );

    if Op::apply(cpu.regs[rs], imm) {
        cp0::except(cpu, cp0::Exception::Trap);
    }
}

pub fn trap_r_type<Op: TrapOperator>(cpu: &mut Cpu) {
    let rs = ((cpu.opcode[0] >> 21) & 31) as usize;
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;

    trace!(
        "{:08X}: {} {}, {}",
        cpu.pc[0],
        Op::NAME,
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
    );

    if Op::apply(cpu.regs[rs], cpu.regs[rt]) {
        cp0::except(cpu, cp0::Exception::Trap);
    }
}
