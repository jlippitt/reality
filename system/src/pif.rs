use super::memory::Memory;
use crate::cpu::Size;
use tracing::trace;

const PIF_DATA_SIZE: usize = 2048;

pub struct Pif {
    mem: Memory,
    rom_locked: bool,
}

impl Pif {
    pub fn new(data: Vec<u8>) -> Self {
        assert!(data.len() == PIF_DATA_SIZE);
        let mut mem: Memory = data.into();
        mem.write(0x7e4, 0u32);
        mem.write(0x7ff, 0u8);
        Self {
            mem,
            rom_locked: false,
        }
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        if address < 0x7c0 && self.rom_locked {
            panic!("Read from locked PIF ROM: {:08X}", address);
        }

        self.mem.read(address)
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) {
        if address < 0x7c0 {
            panic!("Write to PIF ROM: {:08X}", address);
        }

        self.mem.write(address, value);

        let cmd: u8 = self.mem.read(0x7ff);

        if cmd == 0 {
            return;
        }

        match cmd {
            0x10 => {
                self.rom_locked = true;
                trace!("PIF ROM locked");
            }
            _ => todo!("PIF command: {:02X}", cmd),
        }

        self.mem.write(0x7ff, 0u8);
    }
}
