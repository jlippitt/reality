pub use error::Exception;
pub use ex::cop0;
pub use regs::TagLo;

use super::{Bus, Cpu, DcOperation};
use error::ExceptionStage;
use regs::{Regs, REG_NAMES};
use tlb::Tlb;
use tracing::{debug, trace, warn};

const TIMER_INT: u8 = 0x80;
const SOFTWARE_INT: u8 = 0x03;

mod error;
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

    pub fn cp1_usable(&self) -> bool {
        self.regs.status.cu1()
    }

    pub fn cp2_usable(&self) -> bool {
        self.regs.status.cu2()
    }

    pub fn is_fr(&self) -> bool {
        self.regs.status.fr()
    }

    pub fn tag_lo(&self) -> TagLo {
        self.regs.tag_lo
    }

    pub fn read_reg(&mut self, reg: usize) -> i64 {
        match reg {
            0 => u32::from(self.regs.index) as i32 as i64,
            2 => u32::from(self.regs.entry_lo0) as i32 as i64,
            3 => u32::from(self.regs.entry_lo1) as i32 as i64,
            4 => u32::from(self.regs.context) as i32 as i64,
            5 => u32::from(self.regs.page_mask) as i32 as i64,
            6 => u32::from(self.regs.wired) as i32 as i64,
            8 => self.regs.bad_vaddr as i32 as i64,
            9 => self.regs.count as i32 as i64,
            10 => u64::from(self.regs.entry_hi) as i64,
            11 => self.regs.compare as i32 as i64,
            12 => u32::from(self.regs.status) as i32 as i64,
            13 => u32::from(self.regs.cause) as i32 as i64,
            14 => self.regs.epc as i32 as i64,
            16 => u32::from(self.regs.config) as i32 as i64,
            17 => self.regs.ll_addr as i32 as i64,
            20 => u64::from(self.regs.x_context) as i64,
            29 => self.regs.tag_hi as i32 as i64,
            30 => self.regs.error_epc as i32 as i64,
            _ => todo!("CP0 Register Read: {:?}", reg),
        }
    }

    pub fn write_reg(&mut self, reg: usize, value: i64) {
        match reg {
            0 => {
                self.regs.index = (value as u32 & 0x8000_003f).into();
                trace!("  Index: {:?}", self.regs.index);
            }
            2 => {
                self.regs.entry_lo0 = (value as u32 & 0x3fff_ffff).into();
                trace!("  EntryLo0: {:?}", self.regs.entry_lo0);
            }
            3 => {
                self.regs.entry_lo1 = (value as u32 & 0x3fff_ffff).into();
                trace!("  EntryLo1: {:?}", self.regs.entry_lo1);
            }
            4 => {
                self.regs.context = (value as u32).into();
                trace!("  Context: {:?}", self.regs.context);
            }
            5 => {
                self.regs.page_mask = (value as u32 & 0x01ff_e000).into();
                trace!("  PageMask: {:?}", self.regs.page_mask);
            }
            6 => {
                self.regs.wired = (value as u32 & 0x0000_003f).into();
                trace!("  Wired: {:?}", self.regs.wired);
            }
            9 => {
                self.regs.count = value as u32;
                trace!("  Count: {:?}", self.regs.count);
            }
            10 => {
                self.regs.entry_hi = (value as u64 & 0xc000_00ff_ffff_e0ff).into();
                trace!("  EntryHi: {:?}", self.regs.entry_hi);
            }
            11 => {
                self.regs.compare = value as u32;
                trace!("  Compare: {:?}", self.regs.compare);

                let prev_ip = self.regs.cause.ip();
                self.regs.cause.set_ip(prev_ip & !TIMER_INT);

                if (prev_ip & TIMER_INT) != 0 {
                    debug!("CP0 Timer Interrupt Cleared");
                }
            }
            12 => {
                self.regs.status = (value as u32).into();
                trace!("  Status: {:?}", self.regs.status);
                assert_eq!(0, self.regs.status.ksu(), "Only kernel mode is supported");
                assert!(!self.regs.status.rp(), "Low power mode is not supported");

                if self.regs.status.kx() {
                    warn!("Only 32-bit addressing is supported");
                }

                if self.regs.status.ds() != 0 {
                    warn!("CPU diagnostics are not supported");
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
            16 => {
                self.regs.config = (0x7006_6460 | (value as u32 & 0x0f00_800f)).into();
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
            18 => {
                self.regs.watch_lo = (value as u32).into();
                trace!("  WatchLo: {:?}", self.regs.watch_lo);
                assert!(!self.regs.watch_lo.read());
                assert!(!self.regs.watch_lo.write());
            }
            19 => {
                self.regs.watch_hi = (value as u32).into();
                trace!("  WatchHi: {:?}", self.regs.watch_hi);
            }
            20 => {
                self.regs.x_context = (value as u64).into();
                trace!("  XContext: {:?}", self.regs.x_context);
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
    let regs = &mut cpu.cp0.regs;

    regs.count = regs.count.wrapping_add(1);

    if regs.count == regs.compare {
        regs.cause.set_ip(regs.cause.ip() | TIMER_INT);
        debug!("CP0 Timer Interrupt Raised");
    }

    if !regs.status.ie() || regs.status.erl() || regs.status.exl() {
        return;
    }

    let pending = (regs.cause.ip() & (TIMER_INT | SOFTWARE_INT)) | bus.poll();
    regs.cause.set_ip(pending);

    let active = pending & regs.status.im();

    if active == 0 {
        return;
    }

    except(cpu, Exception::Interrupt);
}

pub fn except(cpu: &mut Cpu, ex: Exception) {
    let regs = &mut cpu.cp0.regs;

    debug!("-- Exception: {:?} --", ex);
    let details = ex.process(regs);
    regs.cause.set_exc_code(details.code);
    regs.cause.set_ce(details.ce);
    cpu.pc = 0x8000_0000 | details.vector;

    let epc = match details.stage {
        ExceptionStage::DC => {
            let epc = if cpu.dc.delay {
                cpu.dc.pc.wrapping_sub(4)
            } else {
                cpu.dc.pc
            };
            regs.cause.set_bd(cpu.dc.delay);
            cpu.dc.pc = cpu.pc;
            cpu.dc.op = DcOperation::Nop;
            cpu.ex.pc = cpu.pc;
            cpu.ex.word = 0;
            cpu.rf.pc = cpu.pc;
            cpu.rf.active = false;
            epc
        }
        ExceptionStage::EX => {
            let epc = if cpu.ex.delay {
                cpu.ex.pc.wrapping_sub(4)
            } else {
                cpu.ex.pc
            };
            regs.cause.set_bd(cpu.ex.delay);
            cpu.ex.pc = cpu.pc;
            cpu.ex.word = 0;
            cpu.rf.pc = cpu.pc;
            cpu.rf.active = false;
            epc
        }
        ExceptionStage::RF => {
            let epc = if cpu.rf.delay {
                cpu.rf.pc.wrapping_sub(4)
            } else {
                cpu.rf.pc
            };
            regs.cause.set_bd(cpu.rf.delay);
            cpu.rf.pc = cpu.pc;
            cpu.rf.active = false;
            epc
        }
    };

    if details.error {
        let nested = regs.status.erl();
        regs.status.set_erl(true);
        trace!("  Status: {:?}", regs.status);
        trace!("  Cause: {:?}", regs.cause);

        if !nested {
            regs.error_epc = epc;
            trace!("  ErrorEPC: {:08X}", regs.error_epc);
        }
    } else {
        let nested = regs.status.exl();
        regs.status.set_exl(true);
        trace!("  Status: {:?}", regs.status);
        trace!("  Cause: {:?}", regs.cause);

        if !nested {
            regs.epc = epc;
            trace!("  EPC: {:08X}", regs.epc);
        }
    };
}
