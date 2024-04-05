use crate::memory::Size;
use cache::{DCache, DCacheLine, ICache};
use cp0::Cp0;
use cp1::Cp1;
use dc::DcState;
use tracing::trace;

mod cache;
mod cp0;
mod cp1;
mod dc;
mod ex;

const COLD_RESET_VECTOR: u32 = 0xbfc0_0000;

#[derive(Default)]
struct RfState {
    pc: u32,
    word: u32,
}

#[derive(Default)]
struct ExState {
    pc: u32,
    word: u32,
}

enum WbOperation {
    Cp0RegWrite { reg: usize, value: i64 },
    Cp1ControlRegWrite { reg: usize, value: u32 },
}

struct WbState {
    reg: usize,
    value: i64,
    op: Option<WbOperation>,
}

pub trait Bus {
    fn read_single<T: Size>(&self, address: u32) -> T;
    fn write_single<T: Size>(&mut self, address: u32, value: T);
    fn read_block(&self, address: u32, data: &mut [u32]);
    fn write_block(&mut self, address: u32, data: &[u32]);
    fn poll(&self) -> u8;
}

pub struct Cpu {
    wb: WbState,
    dc: DcState,
    ex: ExState,
    rf: RfState,
    pc: u32,
    hi: i64,
    lo: i64,
    ll_bit: bool,
    regs: [i64; 64],
    cp0: Cp0,
    cp1: Cp1,
    icache: ICache,
    dcache: DCache,
}

impl Cpu {
    const REG_NAMES: [&'static str; 32] = [
        "ZERO", "AT", "V0", "V1", "A0", "A1", "A2", "A3", "T0", "T1", "T2", "T3", "T4", "T5", "T6",
        "T7", "S0", "S1", "S2", "S3", "S4", "S5", "S6", "S7", "T8", "T9", "K0", "K1", "GP", "SP",
        "FP", "RA",
    ];

    pub fn new() -> Self {
        Self {
            rf: Default::default(),
            ex: Default::default(),
            dc: DcState::Nop,
            wb: WbState {
                reg: 0,
                value: 0,
                op: None,
            },
            pc: COLD_RESET_VECTOR,
            hi: 0,
            lo: 0,
            ll_bit: false,
            regs: [0; 64],
            cp0: Cp0::new(),
            cp1: Cp1::new(),
            icache: ICache::new(),
            dcache: DCache::new(),
        }
    }

    pub fn step(&mut self, bus: &mut impl Bus) {
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

        if let Some(op) = &self.wb.op {
            match *op {
                WbOperation::Cp0RegWrite { reg, value } => {
                    self.cp0.write_reg(reg, value);
                }
                WbOperation::Cp1ControlRegWrite { reg, value } => {
                    self.cp1.write_control_reg(reg, value);
                }
            }
        }

        // DC
        dc::execute(self, bus);

        // The official documentation says that:
        // > When an NMI or interrupt exception occurs, all pipeline stages
        // > except the WB are aborted
        // This implies restarting from the DC stage. However, the instruction
        // in the DC stage will have already passed through the EX stage, and
        // this could lead to weird bugs if care isn't taken. So, for now we
        // check for interrupts just before the EX stage, pending further view
        // at a later date.
        // TODO: Re-review this decision
        cp0::step(self, bus);

        // EX
        if self.ex.word != 0 {
            // Operand forwarding from DC stage
            let tmp = self.regs[self.wb.reg];
            self.regs[self.wb.reg] = self.wb.value;
            self.regs[0] = 0;
            self.dc = ex::execute(self, self.ex.pc, self.ex.word);
            self.regs[self.wb.reg] = tmp;
        } else {
            trace!("{:08X}: NOP", self.ex.pc);
            self.dc = DcState::Nop;
        }

        // RF
        self.ex = ExState {
            pc: self.rf.pc,
            word: self.rf.word,
        };

        // IC
        self.rf = RfState {
            pc: self.pc,
            word: self.read_opcode(bus, self.pc),
        };

        self.pc = self.pc.wrapping_add(4);
    }

    fn branch<const LIKELY: bool>(&mut self, condition: bool, offset: i64) {
        if condition {
            trace!("Branch taken");
            self.pc = (self.rf.pc as i64).wrapping_add(offset) as u32;
        } else {
            trace!("Branch not taken");

            if LIKELY {
                self.rf.word = 0;
            }
        }
    }

    fn read<T: Size>(&mut self, bus: &mut impl Bus, address: u32) -> T {
        let segment = address >> 29;

        if (segment & 6) != 4 {
            todo!("TLB lookups");
        }

        if segment == 4 {
            return self.dcache.read(address & 0x1fff_ffff, |line| {
                Self::dcache_reload(bus, line, address)
            });
        }

        bus.read_single(address & 0x1fff_ffff)
    }

    fn write<T: Size>(&mut self, bus: &mut impl Bus, address: u32, value: T) {
        let segment = address >> 29;

        if (segment & 6) != 4 {
            todo!("TLB lookups");
        }

        if segment == 4 {
            return self.dcache.write(address & 0x1fff_ffff, value, |line| {
                Self::dcache_reload(bus, line, address)
            });
        }

        bus.write_single(address & 0x1fff_ffff, value);
    }

    fn read_dword(&mut self, bus: &mut impl Bus, address: u32) -> u64 {
        let segment = address >> 29;

        if (segment & 6) != 4 {
            todo!("TLB lookups");
        }

        let mut dword = [0u32; 2];

        if segment == 4 {
            self.dcache
                .read_block(address & 0x1fff_ffff, &mut dword, |line| {
                    Self::dcache_reload(bus, line, address)
                });
        } else {
            bus.read_block(address & 0x1fff_ffff, &mut dword);
        }

        ((dword[0] as u64) << 32) | (dword[1] as u64)
    }

    fn write_dword(&mut self, bus: &mut impl Bus, address: u32, value: u64) {
        let segment = address >> 29;

        if (segment & 6) != 4 {
            todo!("TLB lookups");
        }

        let dword = [(value >> 32) as u32, value as u32];

        if segment == 4 {
            return self
                .dcache
                .write_block(address & 0x1fff_ffff, &dword, |line| {
                    Self::dcache_reload(bus, line, address)
                });
        }

        bus.write_block(address & 0x1fff_ffff, &dword);
    }

    fn read_opcode(&mut self, bus: &mut impl Bus, address: u32) -> u32 {
        let segment = address >> 29;

        if (segment & 6) != 4 {
            todo!("TLB lookups");
        }

        if segment == 4 {
            return self.icache.read(address & 0x1fff_ffff, |line| {
                bus.read_block(address & 0x1fff_ffe0, line.data_mut());
            });
        }

        bus.read_single(address & 0x1fff_ffff)
    }

    fn dcache_reload(bus: &mut impl Bus, line: &mut DCacheLine, address: u32) {
        // TODO: Timing
        if line.is_dirty() {
            bus.write_block(((line.ptag() & !1) << 12) | (address & 0x1ff0), line.data());
        }

        bus.read_block(address & 0x1fff_fff0, line.data_mut());
    }
}
