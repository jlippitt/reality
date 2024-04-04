pub use ex::{cache, cop0};

use super::{Cpu, DcState};
use regs::{Regs, REG_NAMES};
use tracing::trace;

mod ex;
mod regs;

#[derive(Debug)]
pub struct Cp0 {
    regs: Regs,
}

impl Cp0 {
    pub const REG_NAMES: [&'static str; 32] = REG_NAMES;
    pub const LL_ADDR: usize = 17;

    pub fn new() -> Self {
        Self {
            regs: Regs::default(),
        }
    }

    pub fn is_fr(&self) -> bool {
        self.regs.status.fr()
    }

    pub fn read_reg(&mut self, reg: usize) -> i64 {
        match reg {
            9 => self.regs.count as i64,
            11 => self.regs.compare as i64,
            12 => u32::from(self.regs.status) as i64,
            13 => u32::from(self.regs.cause) as i64,
            14 => self.regs.epc as i64,
            17 => self.regs.ll_addr as i64,
            29 => self.regs.tag_hi as i64,
            30 => self.regs.error_epc as i64,
            _ => todo!("CP0 Register Read: {:?}", reg),
        }
    }

    pub fn write_reg(&mut self, reg: usize, value: i64) {
        match reg {
            9 => {
                self.regs.count = value as u32;
                trace!("  Count: {:?}", self.regs.count);
            }
            11 => {
                self.regs.compare = value as u32;
                trace!("  Compare: {:?}", self.regs.compare);
            }
            12 => {
                self.regs.status = (value as u32).into();
                trace!("  Status: {:?}", self.regs.status);
                assert_eq!(0, self.regs.status.ksu(), "Only kernel mode is supported");
                assert!(
                    !self.regs.status.kx(),
                    "Only 32-bit addressing is supported"
                );
                assert_eq!(0, self.regs.status.ds(), "Diagnostics are not supported");
                assert!(!self.regs.status.rp(), "Low power mode is not supported");

                if self.regs.status.im() != 0 {
                    todo!("Interrupt checks");
                }
            }
            13 => {
                self.regs.cause = (value as u32).into();
                trace!("  Cause: {:?}", self.regs.cause);

                if self.regs.cause.ip() != 0 {
                    todo!("Manual interrupts");
                }
            }
            14 => {
                self.regs.epc = value as u32;
                trace!("  EPC: {:08X}", self.regs.epc);
            }
            // TOOD: This register has special behaviour when read back
            16 => {
                self.regs.config = (value as u32).into();
                trace!("  Config: {:?}", self.regs.config);
                assert_ne!(2, self.regs.config.k0(), "Uncached KSEG0 is not supported");
                assert!(self.regs.config.be(), "Little-endian mode is not supported");
                assert_eq!(
                    0,
                    self.regs.config.ep(),
                    "Only the default transfer data pattern is supported"
                );
            }
            17 => {
                self.regs.ll_addr = value as u32;
                trace!("  LLAddr: {:08X}", self.regs.ll_addr);
            }
            // TOOD: This register has special behaviour when read back
            28 => {
                self.regs.tag_lo = (value as u32).into();
                trace!("  TagLo: {:?}", self.regs.tag_lo);
                assert_eq!(
                    0,
                    value & 0xf000_003f,
                    "Bits 0-5 and 28-31 must be written as zero"
                );
            }
            29 => {
                self.regs.tag_hi = value as u32;
                trace!("  TagHi: {:08X}", self.regs.tag_hi);
                assert_eq!(0, self.regs.tag_hi);
            }
            30 => {
                self.regs.error_epc = value as u32;
                trace!("  ErrorEPC: {:08X}", self.regs.error_epc);
            }
            _ => todo!(
                "CP0 Register Write: {} <= {:016X}",
                Self::REG_NAMES[reg],
                value
            ),
        }
    }
}
