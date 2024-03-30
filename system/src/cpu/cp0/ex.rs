use super::Cp0Register;
use super::{Cpu, DcState};

pub fn mtc0(cpu: &mut Cpu, word: u32) -> DcState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = Cp0Register::from((word >> 11) & 31);

    println!(
        "{:08X}: MTC0 {}, {:?}",
        cpu.pc_debug,
        Cpu::REG_NAMES[rt],
        rd
    );

    DcState::Cp0Write {
        reg: rd,
        value: cpu.regs[rt],
    }
}
