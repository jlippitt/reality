use super::Cp0Register;
use super::{Cpu, DcState};

pub fn mtc0(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = Cp0Register::from((word >> 11) & 31);

    println!("{:08X}: MTC0 {}, {:?}", pc, Cpu::REG_NAMES[rt], rd);

    DcState::Cp0Write {
        reg: rd,
        value: cpu.regs[rt],
    }
}
