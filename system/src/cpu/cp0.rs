pub use ex::cop0;
pub use regs::TagLo;

use super::{Bus, Cpu, DcState};
use regs::{Regs, REG_NAMES};
use tlb::Tlb;
use tracing::{trace, warn};

const EXCEPTION_VECTOR: u32 = 0x8000_0180;

mod ex;
mod regs;
mod tlb;

#[derive(Debug)]
pub struct Cp0 {
    regs: Regs,
    tlb: Tlb,
}

impl Cp0 {
    pub const REG_NAMES: [&'static str; 32] = REG_NAMES;
    pub const LL_ADDR: usize = 17;

    pub fn new() -> Self {
        Self {
            regs: Regs::default(),
            tlb: Tlb::new(),
        }
    }

    pub fn is_fr(&self) -> bool {
        self.regs.status.fr()
    }

    pub fn tag_lo(&self) -> TagLo {
        self.regs.tag_lo
    }

    pub fn read_reg(&mut self, reg: usize) -> i64 {
        match reg {
            0 => u32::from(self.regs.index) as i64,
            2 => u32::from(self.regs.entry_lo0) as i64,
            3 => u32::from(self.regs.entry_lo1) as i64,
            5 => u32::from(self.regs.page_mask) as i64,
            9 => self.regs.count as i64,
            10 => u32::from(self.regs.entry_hi) as i64,
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
            0 => {
                self.regs.index = (value as u32).into();
                trace!("  Index: {:?}", self.regs.index);
            }
            2 => {
                self.regs.entry_lo0 = (value as u32).into();
                trace!("  EntryLo0: {:?}", self.regs.entry_lo0);
            }
            3 => {
                self.regs.entry_lo1 = (value as u32).into();
                trace!("  EntryLo1: {:?}", self.regs.entry_lo1);
            }
            5 => {
                self.regs.page_mask = (value as u32).into();
                trace!("  PageMask: {:?}", self.regs.page_mask);
            }
            9 => {
                self.regs.count = value as u32;
                trace!("  Count: {:?}", self.regs.count);
            }
            10 => {
                self.regs.entry_hi = (value as u32).into();
                trace!("  EntryHi: {:?}", self.regs.entry_hi);
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
                    warn!("TODO: Interrupt checks");
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

pub fn step(cpu: &mut Cpu, bus: &impl Bus) {
    // TODO: Counter update/check
    let regs = &mut cpu.cp0.regs;

    if !regs.status.ie() || regs.status.erl() || regs.status.exl() {
        return;
    }

    let pending = (regs.cause.ip() & 0x83) | bus.poll();
    regs.cause.set_ip(pending);

    let active = pending & regs.status.im();

    if active == 0 {
        return;
    }

    trace!("-- Exception: {:08b} --", active);

    regs.status.set_exl(true);
    regs.cause.set_exc_code(0); // 0 = Interrupt
    regs.cause.set_bd(cpu.delay);
    regs.epc = if cpu.delay {
        cpu.ex.pc.wrapping_sub(4)
    } else {
        cpu.ex.pc
    };
    cpu.pc = EXCEPTION_VECTOR;
    cpu.ex.pc = cpu.pc;
    cpu.ex.word = 0;
    cpu.rf.pc = cpu.pc;
    cpu.rf.word = 0;

    trace!("  Status: {:?}", regs.status);
    trace!("  Cause: {:?}", regs.cause);
    trace!("  EPC: {:08X}", regs.epc);
}
