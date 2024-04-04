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

    pub fn cop0(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
        match (word >> 21) & 31 {
            0o00 => ex::mfc0(cpu, pc, word),
            0o04 => ex::mtc0(cpu, pc, word),
            0o20..=0o37 => match word & 63 {
                0o30 => ex::eret(cpu, pc),
                func => todo!("CPU COP0 Function '{:02o}' at {:08X}", func, pc),
            },
            opcode => todo!("CPU COP0 Opcode '{:02o}' at {:08X}", opcode, pc),
        }
    }

    pub fn cache(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
        const CACHE_OP_NAMES: [char; 8] = ['?', '?', 'P', '?', '?', '?', '?', '?'];
        const CACHE_NAMES: [char; 4] = ['I', 'D', '?', '?'];

        let base = ((word >> 21) & 31) as usize;
        let op = (word >> 16) & 31;
        let offset = (word & 0xffff) as i16;

        trace!(
            "{:08X}: CACHE {}{}, {}({})",
            pc,
            CACHE_OP_NAMES[(op >> 2) as usize],
            CACHE_NAMES[(op & 3) as usize],
            offset,
            Cpu::REG_NAMES[base]
        );

        let address = cpu.regs[base].wrapping_add(offset as i64) as u32;

        match (word >> 16) & 31 {
            0b01000 => {
                let tag = &cpu.cp0.regs.tag_lo;
                let ptag = tag.ptag_lo();
                let valid = (tag.pstate() & 0b10) != 0;
                cpu.icache.index_store_tag(address, ptag, valid);
            }
            0b01001 => {
                let tag = &cpu.cp0.regs.tag_lo;
                let ptag = tag.ptag_lo();
                let valid = (tag.pstate() & 0b10) != 0;
                let dirty = (tag.pstate() & 0b01) != 0;
                cpu.dcache.index_store_tag(address, ptag, valid, dirty);
            }
            op => todo!("Cache Operation: {:05b}", op),
        }

        DcState::Nop
    }
}
