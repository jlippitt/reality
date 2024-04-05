use crate::memory::{Memory, Size};
use tracing::{trace, warn};

const PIF_DATA_SIZE: usize = 2048;

pub struct Pif {
    mem: Memory,
    rom_locked: bool,
}

impl Pif {
    pub fn new(data: Vec<u8>) -> Self {
        assert!(data.len() == PIF_DATA_SIZE);

        let mut mem = Memory::from_bytes(&data);

        // This data is written by PIF upon startup
        // (byte 1 is the seed for the IPL3 CRC check)
        mem.write(0x7e4, 0x0000_3f00u32);

        // The initial command byte is always set to 0
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

        // Impossible to write command byte at these addresses
        if address < 0x7fc {
            return;
        }

        // Interpret PIF command
        let cmd = self.mem.read::<u8>(0x7ff) & 0x7b;

        if cmd == 0 {
            return;
        }

        let mut result: u8 = 0;

        if (cmd & 0x01) != 0 {
            todo!("PIF Joybus Configure");
        }

        if (cmd & 0x02) != 0 {
            todo!("PIF Challenge/Response");
        }

        if (cmd & 0x08) != 0 {
            warn!("TODO: PIF Terminate Boot Process");
        }

        if (cmd & 0x10) != 0 {
            self.rom_locked = true;
            trace!("PIF ROM locked");
        }

        if (cmd & 0x20) != 0 {
            // TODO: Timing?
            result |= 0x80;
        }

        // 0x40 (bit 6) is a NOP

        self.mem.write(0x7ff, result);
    }
}
