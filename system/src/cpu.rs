use bytemuck::Pod;
use cp0::{Cp0, Cp0Register};
use std::mem;

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
    LoadWord { reg: usize, addr: u32 },
    Nop,
}

enum WbOperation {
    Cp0Write { reg: Cp0Register, value: i64 },
}

struct WbState {
    reg: usize,
    value: i64,
    op: Option<WbOperation>,
}

pub trait Size: Pod {
    fn from_u32(value: u32) -> Self;
    fn to_u32(self) -> u32;
}

impl Size for u8 {
    fn from_u32(value: u32) -> Self {
        value as Self
    }

    fn to_u32(self) -> u32 {
        self as u32
    }
}

impl Size for u16 {
    fn from_u32(value: u32) -> Self {
        value as Self
    }

    fn to_u32(self) -> u32 {
        self as u32
    }
}

impl Size for u32 {
    fn from_u32(value: u32) -> Self {
        value as Self
    }

    fn to_u32(self) -> u32 {
        self
    }
}

pub trait Bus {
    fn read_single<T: Size>(&self, address: u32) -> T;
}

pub struct Cpu {
    wb: WbState,
    dc: DcState,
    ex: ExState,
    rf: RfState,
    pc: u32,
    regs: [i64; 32],
    cp0: Cp0,
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
            regs: [0; 32],
            cp0: Cp0::new(),
        }
    }

    pub fn step(&mut self, bus: &mut impl Bus) {
        // WB
        self.regs[self.wb.reg] = self.wb.value;
        self.regs[0] = 0;

        if self.wb.reg != 0 {
            println!("  {}: {:016X}", Self::REG_NAMES[self.wb.reg], self.wb.value);
        }

        if let Some(op) = &self.wb.op {
            match *op {
                WbOperation::Cp0Write { reg, value } => {
                    self.cp0.write_reg(reg, value);
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
            DcState::LoadWord { reg, addr } => {
                // TODO: Stall cycles
                self.wb.reg = reg;
                self.wb.value = self.read::<u32>(bus, addr) as i64;
                println!("  {:08X} => {:08X}", addr, self.wb.value);
                self.wb.op = None;
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
            println!("{:08X}: NOP", self.ex.pc);
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
            word: self.read(bus, self.pc),
        };

        self.pc = self.pc.wrapping_add(4);
    }

    fn read<T: Size>(&self, bus: &mut impl Bus, address: u32) -> T {
        let segment = address >> 29;

        if (segment & 6) != 4 {
            todo!("TLB lookups");
        }

        if segment == 4 {
            todo!("Cached reads");
        }

        if mem::size_of::<T>() > 4 {
            todo!("Block reads");
        }

        bus.read_single(address & 0x1fff_ffff)
    }
}
