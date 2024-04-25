use crate::memory::Size;
use cache::ICache;
use cp0::{Cp0, Exception};
use cp1::Cp1;
use dc::DcOperation;
use tracing::trace;

#[cfg(feature = "dcache")]
use cache::{DCache, DCacheLine};

mod cache;
mod cp0;
mod cp1;
mod dc;
mod ex;

const COLD_RESET_VECTOR: u32 = 0xbfc0_0000;
const IPL3_START: u32 = 0xA4000040;

// Fixed values taken from Cen64
// TODO: One day these will be dynamic
const RW_SINGLE_WORD_DELAY: u64 = 38;
const REFRESH_ICACHE_DELAY: u64 = 48;

#[cfg(feature = "dcache")]
const REFRESH_DCACHE_DELAY: u64 = 44;

// Try to guess average DCache hit rate
#[cfg(not(feature = "dcache"))]
const RW_SINGLE_WORD_DCACHE_DELAY: u64 = 4;

#[derive(Default)]
struct RfState {
    pc: u32,
    delay: bool,
    active: bool,
}

#[derive(Default)]
struct ExState {
    pc: u32,
    delay: bool,
    word: u32,
}

#[derive(Default)]
struct DcState {
    pc: u32,
    delay: bool,
    op: DcOperation,
}

#[derive(Default)]
struct WbState {
    reg: usize,
    value: i64,
}

#[cfg(feature = "profiling")]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Stats {
    pub instruction_cycles: u64,
    pub stall_cycles: u64,
    pub busy_wait_cycles: u64,
}

pub trait Bus {
    fn read_single<T: Size>(&self, address: u32) -> T;
    fn write_single<T: Size>(&mut self, address: u32, value: T);
    fn read_block<T: Size>(&self, address: u32, data: &mut [T]);
    fn write_block<T: Size>(&mut self, address: u32, data: &[T]);
    fn poll(&self) -> u8;
}

pub struct Cpu {
    stall: u64,
    busy_wait: bool,
    wb: WbState,
    dc: DcState,
    ex: ExState,
    rf: RfState,
    pc: u32,
    regs: [i64; 64],
    hi: i64,
    lo: i64,
    ll_bit: bool,
    cp0: Cp0,
    cp1: Cp1,
    icache: ICache,
    #[cfg(feature = "dcache")]
    dcache: DCache,
    #[cfg(feature = "profiling")]
    stats: Stats,
}

impl Cpu {
    const REG_NAMES: [&'static str; 32] = [
        "ZERO", "AT", "V0", "V1", "A0", "A1", "A2", "A3", "T0", "T1", "T2", "T3", "T4", "T5", "T6",
        "T7", "S0", "S1", "S2", "S3", "S4", "S5", "S6", "S7", "T8", "T9", "K0", "K1", "GP", "SP",
        "FP", "RA",
    ];

    pub fn new(skip_pif_rom: bool) -> Self {
        let mut regs = [0; 64];

        let pc = if skip_pif_rom {
            regs[20] = 1;
            regs[22] = 0x3f;
            regs[29] = 0xffff_ffff_a400_1ff0u64 as i64;
            IPL3_START
        } else {
            COLD_RESET_VECTOR
        };

        Self {
            stall: 0,
            busy_wait: false,
            wb: WbState::default(),
            dc: DcState::default(),
            ex: ExState::default(),
            rf: RfState::default(),
            pc,
            regs,
            hi: 0,
            lo: 0,
            ll_bit: false,
            cp0: Cp0::new(),
            cp1: Cp1::new(),
            icache: ICache::new(),
            #[cfg(feature = "dcache")]
            dcache: DCache::new(),
            #[cfg(feature = "profiling")]
            stats: Stats::default(),
        }
    }

    #[cfg(feature = "profiling")]
    pub fn stats(&self) -> &Stats {
        &self.stats
    }

    #[cfg(feature = "profiling")]
    pub fn reset_stats(&mut self) {
        self.stats = Stats::default();
    }

    // 1. Always inlined
    #[inline(always)]
    pub fn step(&mut self, bus: &mut impl Bus) {
        if self.stall > 0 {
            #[cfg(feature = "profiling")]
            {
                self.stats.stall_cycles += 1;
            }

            self.stall -= 1;
            return;
        }

        self.step_inner(bus);
    }

    // 2. Let the compiler decide whether to inline
    fn step_inner(&mut self, bus: &mut impl Bus) {
        if self.busy_wait {
            #[cfg(feature = "profiling")]
            {
                self.stats.busy_wait_cycles += 1;
            }

            cp0::step(self, bus);
            return;
        }

        self.step_cycle(bus);
    }

    // 3. Probably won't be inlined
    fn step_cycle(&mut self, bus: &mut impl Bus) {
        #[cfg(feature = "profiling")]
        {
            self.stats.instruction_cycles += 1;
        }

        // WB
        self.regs[self.wb.reg] = self.wb.value;
        self.regs[0] = 0;

        if self.wb.reg != 0 {
            if self.wb.reg < Cp1::REG_OFFSET {
                trace!("  {}: {:016X}", Self::REG_NAMES[self.wb.reg], self.wb.value);
            } else {
                trace!(
                    "  F{}: {:016X}",
                    self.wb.reg - Cp1::REG_OFFSET,
                    self.wb.value
                );
            }
        }

        // DC
        cp0::step(self, bus);

        dc::execute(self, bus);

        // EX
        self.dc.op = if self.ex.word != 0 {
            // Operand forwarding from DC stage
            let tmp = self.regs[self.wb.reg];
            self.regs[self.wb.reg] = self.wb.value;
            self.regs[0] = 0;
            let op = ex::execute(self, self.ex.pc, self.ex.word);
            self.regs[self.wb.reg] = tmp;
            op
        } else {
            trace!("{:08X}: NOP", self.ex.pc);
            DcOperation::Nop
        };

        self.dc.pc = self.ex.pc;
        self.dc.delay = self.ex.delay;

        // RF
        self.ex = ExState {
            pc: self.rf.pc,
            delay: self.rf.delay,
            word: if self.rf.active {
                self.read_opcode(bus, self.rf.pc)
            } else {
                0
            },
        };

        // IC
        self.rf = RfState {
            pc: self.pc,
            delay: false,
            active: true,
        };

        self.pc = self.pc.wrapping_add(4);
    }

    fn branch<const LIKELY: bool>(&mut self, condition: bool, offset: i64) {
        if self.ex.delay {
            return;
        }

        self.rf.delay = true;

        if condition {
            trace!("Branch taken");
            self.pc = (self.ex.pc as i64).wrapping_add(offset + 4) as u32;
        } else {
            trace!("Branch not taken");

            if LIKELY {
                self.rf.active = false;
            }
        }
    }

    fn read_data<T: Size>(&mut self, bus: &mut impl Bus, vaddr: u32) -> Option<T> {
        let region = vaddr >> 29;

        if region == 4 {
            let paddr = vaddr & 0x1fff_ffff;

            #[cfg(feature = "dcache")]
            return Some(self.dcache.read(paddr & 0x1fff_ffff, |line| {
                Self::dcache_reload(bus, line, paddr)
            }));

            #[cfg(not(feature = "dcache"))]
            {
                self.stall += RW_SINGLE_WORD_DCACHE_DELAY;
                return Some(bus.read_single(paddr));
            }
        }

        if region == 5 {
            let paddr = vaddr & 0x1fff_ffff;
            self.stall += RW_SINGLE_WORD_DELAY;
            return Some(bus.read_single(paddr));
        }

        self.read_data_tlb(bus, vaddr)
    }

    fn read_data_tlb<T: Size>(&mut self, bus: &mut impl Bus, vaddr: u32) -> Option<T> {
        let Some(result) = self.cp0.translate(vaddr) else {
            cp0::except(
                self,
                Exception::TlbMissLoad(vaddr, false),
                cp0::ExceptionStage::DC,
            );
            return None;
        };

        if !result.valid {
            cp0::except(
                self,
                Exception::TlbMissLoad(vaddr, true),
                cp0::ExceptionStage::DC,
            );
            return None;
        }

        if result.cached {
            #[cfg(feature = "dcache")]
            return self.dcache.read(result.paddr & 0x1fff_ffff, |line| {
                Self::dcache_reload(bus, line, result.paddr)
            });

            #[cfg(not(feature = "dcache"))]
            {
                self.stall += RW_SINGLE_WORD_DCACHE_DELAY;
                return Some(bus.read_single(result.paddr));
            }
        }

        self.stall += RW_SINGLE_WORD_DELAY;
        Some(bus.read_single(result.paddr))
    }

    fn write_data<T: Size>(&mut self, bus: &mut impl Bus, vaddr: u32, value: T) {
        let region = vaddr >> 29;

        if region == 4 {
            let paddr = vaddr & 0x1fff_ffff;

            #[cfg(feature = "dcache")]
            return self
                .dcache
                .write(result.paddr & 0x1fff_ffff, value, |line| {
                    Self::dcache_reload(bus, line, paddr)
                });

            #[cfg(not(feature = "dcache"))]
            {
                self.stall += RW_SINGLE_WORD_DCACHE_DELAY;
                bus.write_single(paddr, value);
                return;
            }
        }

        if region == 5 {
            let paddr = vaddr & 0x1fff_ffff;
            self.stall += RW_SINGLE_WORD_DELAY;
            bus.write_single(paddr, value);
            return;
        }

        self.write_data_tlb(bus, vaddr, value);
    }

    fn write_data_tlb<T: Size>(&mut self, bus: &mut impl Bus, vaddr: u32, value: T) {
        let Some(result) = self.cp0.translate(vaddr) else {
            cp0::except(
                self,
                Exception::TlbMissStore(vaddr, false),
                cp0::ExceptionStage::DC,
            );
            return;
        };

        if !result.valid {
            cp0::except(
                self,
                Exception::TlbMissStore(vaddr, true),
                cp0::ExceptionStage::DC,
            );
            return;
        }

        if !result.writable {
            cp0::except(
                self,
                Exception::TlbModification(vaddr),
                cp0::ExceptionStage::DC,
            );
            return;
        }

        if result.cached {
            #[cfg(feature = "dcache")]
            return self
                .dcache
                .write(result.paddr & 0x1fff_ffff, value, |line| {
                    Self::dcache_reload(bus, line, result.paddr)
                });

            #[cfg(not(feature = "dcache"))]
            {
                self.stall = RW_SINGLE_WORD_DCACHE_DELAY;
                bus.write_single(result.paddr, value);
                return;
            }
        }

        self.stall += RW_SINGLE_WORD_DELAY;
        bus.write_single(result.paddr, value);
    }

    fn read_opcode(&mut self, bus: &mut impl Bus, vaddr: u32) -> u32 {
        let region = vaddr >> 29;

        if region == 4 {
            let paddr = vaddr & 0x1fff_ffff;

            return self.icache.read(vaddr, paddr, |line| {
                self.stall += REFRESH_ICACHE_DELAY;
                bus.read_block(paddr & !0x1f, line.bytes_mut());
            });
        }

        if region == 5 {
            let paddr = vaddr & 0x1fff_ffff;
            self.stall += RW_SINGLE_WORD_DELAY;
            return bus.read_single(paddr);
        }

        self.read_opcode_tlb(bus, vaddr)
    }

    fn read_opcode_tlb(&mut self, bus: &mut impl Bus, vaddr: u32) -> u32 {
        let Some(result) = self.cp0.translate(vaddr) else {
            cp0::except(
                self,
                Exception::TlbMissLoad(vaddr, false),
                cp0::ExceptionStage::RF,
            );
            return 0;
        };

        if !result.valid {
            cp0::except(
                self,
                Exception::TlbMissLoad(vaddr, true),
                cp0::ExceptionStage::RF,
            );
            return 0;
        }

        if result.cached {
            return self.icache.read(vaddr, result.paddr, |line| {
                self.stall += REFRESH_ICACHE_DELAY;
                bus.read_block(result.paddr & !0x1f, line.bytes_mut());
            });
        }

        self.stall += RW_SINGLE_WORD_DELAY;
        bus.read_single(result.paddr)
    }

    #[cfg(feature = "dcache")]
    fn dcache_reload(bus: &mut impl Bus, line: &mut DCacheLine, address: u32) {
        // TODO: Timing
        if line.is_dirty() {
            self.stall += REFRESH_DCACHE_DELAY;
            bus.write_block(
                ((line.ptag() & !1) << 12) | (address & 0x1ff0),
                line.bytes(),
            );
        }

        bus.read_block(address & 0x1fff_fff0, line.bytes_mut());
    }
}
