use crate::memory::Size;
use cp2::Cp2;
use tracing::trace;

mod cp0;
mod cp2;
mod instruction;

pub trait Bus {
    fn read_opcode(&self, address: u32) -> u32;
    fn read_data<T: Size>(&self, address: u32) -> T;
    fn write_data<T: Size>(&mut self, address: u32, value: T);
    fn read_register(&self, index: usize) -> u32;
    fn write_register(&mut self, index: usize, value: u32);
    fn break_(&mut self);
}

pub struct Core {
    opcode: [u32; 2],
    delay: [bool; 2],
    pc: [u32; 3],
    broke: bool,
    regs: [i32; 32],
    cp2: Cp2,
}

impl Core {
    const REG_NAMES: [&'static str; 32] = [
        "ZERO", "AT", "V0", "V1", "A0", "A1", "A2", "A3", "T0", "T1", "T2", "T3", "T4", "T5", "T6",
        "T7", "S0", "S1", "S2", "S3", "S4", "S5", "S6", "S7", "T8", "T9", "K0", "K1", "GP", "SP",
        "FP", "RA",
    ];

    pub fn new() -> Self {
        Self {
            opcode: [0; 2],
            delay: [false; 2],
            pc: [0; 3],
            broke: false,
            regs: [0; 32],
            cp2: Cp2::new(),
        }
    }

    pub fn pc(&self) -> u32 {
        self.pc[2]
    }

    pub fn set_pc(&mut self, value: u32) {
        self.opcode = [0; 2];
        self.delay = [false; 2];
        self.pc = [value & 0x0ffc; 3];
    }

    pub fn step(&mut self, bus: &mut impl Bus) {
        instruction::execute(self, bus);

        if self.broke {
            self.broke = false;
            bus.break_();
            return;
        }

        self.opcode[0] = self.opcode[1];
        self.delay[0] = self.delay[1];
        self.pc[0] = self.pc[1];

        self.opcode[1] = bus.read_opcode(self.pc[2]);
        self.delay[1] = false;
        self.pc[1] = self.pc[2];

        self.pc[2] = self.pc[2].wrapping_add(4) & 0x0ffc;
    }

    fn set_reg(&mut self, reg: usize, value: i32) {
        self.regs[reg] = value;
        self.regs[0] = 0;
        trace!("  {}: {:08X}", Self::REG_NAMES[reg], value);
    }

    fn branch(&mut self, condition: bool, offset: i32) {
        if self.delay[0] {
            return;
        }

        self.delay[1] = true;

        if condition {
            trace!("Branch taken");
            self.pc[2] = (self.pc[0] as i32).wrapping_add(offset + 4) as u32 & 0x0ffc;
        } else {
            trace!("Branch not taken");
        }
    }
}
