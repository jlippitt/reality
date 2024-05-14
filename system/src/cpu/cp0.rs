pub use exception::Exception;
pub use instruction::cop0;
pub use regs::TagLo;
pub use tlb::TlbResult;

use super::{Bus, Cpu};
use regs::{Regs, REG_NAMES};
use tlb::Tlb;
use tracing::{debug, trace, warn};

const TIMER_INT: u8 = 0x80;
const SOFTWARE_INT: u8 = 0x03;

const EXCEPTION_DELAY: u64 = 2;

const RAND_MAX: u32 = 31;

mod exception;
mod instruction;
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
            regs: Regs {
                random: RAND_MAX,
                ..Regs::default()
            },
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

    pub fn translate(&self, vaddr: u32) -> Option<TlbResult> {
        self.tlb.translate(self.regs.entry_hi.asid(), vaddr)
    }

    pub fn read_reg(&mut self, reg: usize) -> i64 {
        match reg {
            0 => u32::from(self.regs.index) as i32 as i64,
            1 => self.regs.random as i32 as i64,
            2 => u32::from(self.regs.entry_lo0) as i32 as i64,
            3 => u32::from(self.regs.entry_lo1) as i32 as i64,
            4 => u64::from(self.regs.context) as i64,
            5 => u32::from(self.regs.page_mask) as i32 as i64,
            6 => self.regs.wired as i32 as i64,
            8 => self.regs.bad_vaddr as i32 as i64,
            9 => self.regs.count as i32 as i64,
            10 => u64::from(self.regs.entry_hi) as i64,
            11 => self.regs.compare as i32 as i64,
            12 => u32::from(self.regs.status) as i32 as i64,
            13 => u32::from(self.regs.cause) as i32 as i64,
            14 => self.regs.epc,
            16 => u32::from(self.regs.config) as i32 as i64,
            17 => self.regs.ll_addr as i64, // Note: No sign-extension
            20 => u64::from(self.regs.x_context) as i64,
            29 => self.regs.tag_hi as i32 as i64,
            30 => self.regs.error_epc,
            _ => todo!("CP0 Register Read: {:?}", reg),
        }
    }

    pub fn write_reg(&mut self, reg: usize, value: i64) {
        match reg {
            0 => {
                self.regs.index = (value as u32 & 0x8000_003f).into();
                trace!("  Index: {:?}", self.regs.index);
            }
            1 => (), // 'Random' is read-only
            2 => {
                self.regs.entry_lo0 = (value as u32 & 0x3fff_ffff).into();
                trace!("  EntryLo0: {:?}", self.regs.entry_lo0);
            }
            3 => {
                self.regs.entry_lo1 = (value as u32 & 0x3fff_ffff).into();
                trace!("  EntryLo1: {:?}", self.regs.entry_lo1);
            }
            4 => {
                self.regs.context = (value as u64 & 0xffff_ffff_ff80_0000).into();
                trace!("  Context: {:?}", self.regs.context);
            }
            5 => {
                self.regs.page_mask = (value as u32 & 0x01ff_e000).into();
                trace!("  PageMask: {:?}", self.regs.page_mask);
            }
            6 => {
                self.regs.wired = (value as u32) & 0x3f;
                self.regs.random = RAND_MAX;
                trace!("  Wired: {:?}", self.regs.wired);
                trace!("  Random: {:?}", self.regs.random);
            }
            8 => (), // 'BadVAddr' is read-only
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
                self.regs.epc = value;
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
                self.regs.x_context = (value as u64 & 0xffff_fffe_0000_0000).into();
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
                self.regs.error_epc = value;
                trace!("  ErrorEPC: {:08X}", self.regs.error_epc);
            }
            _ => todo!(
                "CP0 Register Write: {} <= {:016X}",
                Self::REG_NAMES[reg],
                value
            ),
        }
    }

    pub fn update_counters(&mut self) {
        if self.regs.random == self.regs.wired {
            self.regs.random = RAND_MAX;
        } else {
            self.regs.random = self.regs.random.wrapping_sub(1) & 63;
        }

        self.regs.count = self.regs.count.wrapping_add(1);

        if self.regs.count == self.regs.compare {
            self.regs.cause.set_ip(self.regs.cause.ip() | TIMER_INT);
            debug!("CP0 Timer Interrupt Raised");
        }
    }
}

pub fn handle_interrupt(cpu: &mut Cpu, bus: &impl Bus) -> bool {
    let regs = &mut cpu.cp0.regs;

    if !regs.status.ie() || regs.status.erl() || regs.status.exl() {
        return false;
    }

    let pending = (regs.cause.ip() & (TIMER_INT | SOFTWARE_INT)) | bus.poll();
    regs.cause.set_ip(pending);

    let active = pending & regs.status.im();

    if active == 0 {
        return false;
    }

    except(cpu, Exception::Interrupt);
    true
}

pub fn except(cpu: &mut Cpu, ex: Exception) {
    except_inner(cpu, ex, false)
}

pub fn except_opcode(cpu: &mut Cpu, ex: Exception) {
    except_inner(cpu, ex, true)
}

fn except_inner(cpu: &mut Cpu, ex: Exception, opcode: bool) {
    let regs = &mut cpu.cp0.regs;

    debug!("-- Exception: {:?} --", ex);
    let details = ex.process(regs);
    regs.cause.set_exc_code(details.code);
    regs.cause.set_ce(details.ce);

    let vector = 0x8000_0000 | details.vector;

    let epc = if opcode {
        let delay = has_delay_slot(cpu.opcode[0]);

        let epc = if delay {
            cpu.pc[1].wrapping_sub(4)
        } else {
            cpu.pc[1]
        };

        regs.cause.set_bd(delay);
        cpu.delay[0] = true;
        epc
    } else {
        let epc = if cpu.delay[0] {
            cpu.pc[0].wrapping_sub(4)
        } else {
            cpu.pc[0]
        };

        regs.cause.set_bd(cpu.delay[0]);
        cpu.opcode[0] = 0;
        cpu.delay[0] = false;
        epc
    };

    cpu.opcode[1] = 0;
    cpu.delay[1] = false;
    cpu.pc[2] = vector;

    if details.error {
        let nested = regs.status.erl();
        regs.status.set_erl(true);
        trace!("  Status: {:?}", regs.status);
        trace!("  Cause: {:?}", regs.cause);

        if !nested {
            regs.error_epc = epc as i32 as i64;
            trace!("  ErrorEPC: {:08X}", regs.error_epc);
        }
    } else {
        let nested = regs.status.exl();
        regs.status.set_exl(true);
        trace!("  Status: {:?}", regs.status);
        trace!("  Cause: {:?}", regs.cause);

        if !nested {
            regs.epc = epc as i32 as i64;
            trace!("  EPC: {:08X}", regs.epc);
        }
    };

    cpu.stall += EXCEPTION_DELAY;
}

fn has_delay_slot(word: u32) -> bool {
    let opcode = word >> 26;

    // J, JAL, BEQ, BNE, BLEZ, BNEZ + likely forms
    if (opcode & 0o76) == 0o02 || (opcode & 0o54) == 0o04 {
        return true;
    }

    if opcode == 0o00 {
        // JR, JALR
        if (word & 0o76) == 0o10 {
            return true;
        }
    } else if opcode == 0o01 {
        // BLTZ, BGTZ + likely/linked forms
        if ((word >> 16) & 0o14) == 0o00 {
            return true;
        }
    }

    false
}
