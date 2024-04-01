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
                let tag = TagLo::from(cpu.cp0.regs[Cp0Register::TagLo as usize] as u32);
                let ptag = tag.ptag_lo();
                let valid = (tag.pstate() & 0b10) != 0;
                cpu.icache.index_store_tag(address, ptag, valid);
            }
            0b01001 => {
                let tag = TagLo::from(cpu.cp0.regs[Cp0Register::TagLo as usize] as u32);
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
