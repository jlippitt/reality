use crate::memory::Size;
use cache::{DCache, ICache};
use cp0::{Cp0, Cp0Register};
use tracing::trace;

mod cache;
mod cp0;
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

enum DcState {
    RegWrite { reg: usize, value: i64 },
    Cp0Write { reg: Cp0Register, value: i64 },
    LoadByte { reg: usize, addr: u32 },
    LoadByteUnsigned { reg: usize, addr: u32 },
    LoadHalfword { reg: usize, addr: u32 },
    LoadHalfwordUnsigned { reg: usize, addr: u32 },
    LoadWord { reg: usize, addr: u32 },
    StoreByte { value: u8, addr: u32 },
    StoreHalfword { value: u16, addr: u32 },
    StoreWord { value: u32, addr: u32 },
    MfHi { reg: usize },
    MfLo { reg: usize },
    Nop,
}

enum WbOperation {
    Cp0Write { reg: Cp0Register, value: i64 },
    MfHi { reg: usize },
    MfLo { reg: usize },
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
}

pub struct Cpu {
    wb: WbState,
    dc: DcState,
    ex: ExState,
    rf: RfState,
    pc: u32,
    hi: i64,
    lo: i64,
    regs: [i64; 32],
    cp0: Cp0,
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
            regs: [0; 32],
            cp0: Cp0::new(),
            icache: ICache::new(),
            dcache: DCache::new(),
        }
    }

    pub fn step(&mut self, bus: &mut impl Bus) {
        // WB
        self.regs[self.wb.reg] = self.wb.value;
        self.regs[0] = 0;

        if self.wb.reg != 0 {
            trace!("  {}: {:016X}", Self::REG_NAMES[self.wb.reg], self.wb.value);
        }

        if let Some(op) = &self.wb.op {
            match *op {
                WbOperation::Cp0Write { reg, value } => {
                    self.cp0.write_reg(reg, value);
                }
                WbOperation::MfHi { reg } => {
                    self.regs[reg] = self.hi;
                    trace!("  {}: {:016X}", Self::REG_NAMES[reg], self.hi);
                }
                WbOperation::MfLo { reg } => {
                    self.regs[reg] = self.lo;
                    trace!("  {}: {:016X}", Self::REG_NAMES[reg], self.lo);
                }
            }
        }

        // DC
        match self.dc {
            DcState::RegWrite { reg, value } => {
                self.wb.reg = reg;
                self.wb.value = value;
                self.wb.op = None;
            }
            DcState::Cp0Write { reg, value } => {
                self.wb.reg = 0;
                // self.wb.value doesn't matter
                self.wb.op = Some(WbOperation::Cp0Write { reg, value });
            }
            DcState::LoadByte { reg, addr } => {
                // TODO: Stall cycles
                let value = self.read::<u8>(bus, addr);
                self.wb.reg = reg;
                self.wb.value = value as i8 as i64;
                self.wb.op = None;
                trace!("  [{:08X} => {:02X}]", addr, value);
            }
            DcState::LoadByteUnsigned { reg, addr } => {
                // TODO: Stall cycles
                let value = self.read::<u8>(bus, addr);
                self.wb.reg = reg;
                self.wb.value = value as i64;
                self.wb.op = None;
                trace!("  [{:08X} => {:02X}]", addr, value);
            }
            DcState::LoadHalfword { reg, addr } => {
                // TODO: Stall cycles
                let value = self.read::<u16>(bus, addr) as i16 as i64;
                assert!((addr & 1) == 0);
                self.wb.reg = reg;
                self.wb.value = value as i16 as i64;
                self.wb.op = None;
                trace!("  [{:08X} => {:04X}]", addr, value);
            }
            DcState::LoadHalfwordUnsigned { reg, addr } => {
                // TODO: Stall cycles
                let value = self.read::<u16>(bus, addr);
                assert!((addr & 1) == 0);
                self.wb.reg = reg;
                self.wb.value = value as i64;
                self.wb.op = None;
                trace!("  [{:08X} => {:04X}]", addr, value);
            }
            DcState::LoadWord { reg, addr } => {
                // TODO: Stall cycles
                let value = self.read::<u32>(bus, addr);
                assert!((addr & 3) == 0);
                self.wb.reg = reg;
                self.wb.value = value as i32 as i64;
                self.wb.op = None;
                trace!("  [{:08X} => {:08X}]", addr, value);
            }
            DcState::StoreByte { value, addr } => {
                // TODO: Stall cycles
                self.wb.reg = 0;
                self.wb.op = None;
                trace!("  [{:08X} <= {:02X}]", addr, value);
                self.write(bus, addr, value);
            }
            DcState::StoreHalfword { value, addr } => {
                // TODO: Stall cycles
                assert!((addr & 1) == 0);
                self.wb.reg = 0;
                self.wb.op = None;
                trace!("  [{:08X} <= {:04X}]", addr, value);
                self.write(bus, addr, value);
            }
            DcState::StoreWord { value, addr } => {
                // TODO: Stall cycles
                assert!((addr & 3) == 0);
                self.wb.reg = 0;
                self.wb.op = None;
                trace!("  [{:08X} <= {:08X}]", addr, value);
                self.write(bus, addr, value);
            }
            DcState::MfHi { reg } => {
                self.wb.reg = 0;
                // self.wb.value doesn't matter
                self.wb.op = Some(WbOperation::MfHi { reg });
            }
            DcState::MfLo { reg } => {
                self.wb.reg = 0;
                // self.wb.value doesn't matter
                self.wb.op = Some(WbOperation::MfLo { reg });
            }
            DcState::Nop => {
                self.wb.reg = 0;
                self.wb.op = None;
            }
        }

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

    fn read<T: Size>(&mut self, bus: &mut impl Bus, address: u32) -> T {
        let segment = address >> 29;

        if (segment & 6) != 4 {
            todo!("TLB lookups");
        }

        if segment == 4 {
            return self.dcache.read(address).unwrap_or_else(|| {
                // TODO: Timing
                let mut data = [0u32; 4];
                bus.read_block(address & 0x1fff_ffe0, &mut data);
                self.dcache.insert_line(address, data)
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
            todo!("Cached writes");
        }

        bus.write_single(address & 0x1fff_ffff, value);
    }

    fn read_opcode(&mut self, bus: &mut impl Bus, address: u32) -> u32 {
        let segment = address >> 29;

        if (segment & 6) != 4 {
            todo!("TLB lookups");
        }

        if segment == 4 {
            return self.icache.read(address).unwrap_or_else(|| {
                // TODO: Timing
                let mut data = [0u32; 8];
                bus.read_block(address & 0x1fff_ffe0, &mut data);
                self.icache.insert_line(address, data)
            });
        }

        bus.read_single(address & 0x1fff_ffff)
    }
}
