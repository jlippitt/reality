use super::{Cpu, DcState};
pub use regs::Cp0Register;
use regs::{Cause, Config, Status, TagLo};
use tracing::trace;

mod ex;
mod regs;

#[derive(Debug)]
pub struct Cp0 {
    regs: [i64; 32],
}

impl Cp0 {
    pub fn new() -> Self {
        Self { regs: [0; 32] }
    }

    pub fn write_reg(&mut self, reg: Cp0Register, value: i64) {
        self.regs[reg as usize] = value;

        match reg {
            Cp0Register::Status => {
                let status = Status::from(value as u32);
                trace!("  Status: {:?}", status);
                assert_eq!(0, status.ksu(), "Only kernel mode is supported");
                assert!(!status.kx(), "Only 32-bit addressing is supported");
                assert_eq!(0, status.ds(), "Diagnostics are not supported");
                assert!(!status.rp(), "Low power mode is not supported");

                if status.im() != 0 {
                    todo!("Interrupt checks");
                }
            }
            Cp0Register::Cause => {
                let cause = Cause::from(value as u32);
                trace!("  Cause: {:?}", cause);

                if cause.ip() != 0 {
                    todo!("Manual interrupts");
                }
            }
            // TOOD: This register has special behaviour when read back
            Cp0Register::Config => {
                let config = Config::from(value as u32);
                trace!("  Config: {:?}", config);
                assert_ne!(2, config.k0(), "Uncached KSEG0 is not supported");
                assert!(config.be(), "Little-endian mode is not supported");
                assert_eq!(
                    0,
                    config.ep(),
                    "Only the default transfer data pattern is supported"
                );
            }
            Cp0Register::TagLo => {
                let tag_lo = TagLo::from(value as u32);
                trace!("  TagLo: {:?}", tag_lo);
                assert_eq!(
                    0,
                    value & 0xf000_003f,
                    "Bits 0-5 and 28-31 must be written as zero"
                );
            }
            Cp0Register::TagHi => {
                assert_eq!(0, value);
            }
            Cp0Register::Count | Cp0Register::Compare => {
                trace!("  {:?}: {:08X}", reg, value as u32);
            }
            _ => todo!("Write to {:?}", reg),
        }
    }

    pub fn cop0(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
        match (word >> 21) & 31 {
            0o04 => ex::mtc0(cpu, pc, word),
            opcode => todo!("CPU COP0 Opcode '{:02o}' at {:08X}", opcode, pc),
        }
    }
}
