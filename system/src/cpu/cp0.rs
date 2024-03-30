use super::{Cpu, DcState};
pub use regs::Cp0Register;
use regs::{Config, Status};

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
                println!("  Status: {:?}", status);
                assert_eq!(status.ksu(), 0, "Only kernel mode is supported");
                assert!(!status.kx(), "Only 32-bit addressing is supported");
                assert_eq!(status.ds(), 0, "Diagnostics are not supported");
                assert!(!status.rp(), "Low power mode is not supported");
            }
            // TOOD: This register has special behaviour when read back
            Cp0Register::Config => {
                let config = Config::from(value as u32);
                println!("  Config: {:?}", config);
                assert_ne!(config.k0(), 2, "Uncached KSEG0 is not supported");
                assert!(config.be(), "Little-endian mode is not supported");
                assert_eq!(
                    config.ep(),
                    0,
                    "Only the default transfer data pattern is supported"
                );
            }
            _ => todo!("Write to {:?}", reg),
        }
    }

    pub fn cop0(cpu: &mut Cpu, word: u32) -> DcState {
        match (word >> 21) & 31 {
            0o04 => ex::mtc0(cpu, word),
            opcode => todo!("COP0 Opcode '{:02o}' at {:08X}", opcode, cpu.pc_debug),
        }
    }
}