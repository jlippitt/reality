use super::Joybus;
use crate::memory::{Memory, Size};
use tracing::{trace, warn};

const PIF_DATA_SIZE: usize = 2048;
const PIF_RAM_START: u32 = 0x7c0;

pub struct Pif {
    mem: Memory,
    rom_locked: bool,
}

impl Pif {
    pub fn new(data: Option<Vec<u8>>) -> Self {
        let (mut mem, rom_locked) = if let Some(data) = data {
            assert!(data.len() == PIF_DATA_SIZE);
            (Memory::from_bytes(&data), false)
        } else {
            (Memory::with_byte_len(PIF_DATA_SIZE), true)
        };

        // This data is written by PIF upon startup
        // (byte 1 is the seed for the IPL3 CRC check)
        mem.write(0x7e4, 0x0000_3f00u32);

        // The initial command byte is always set to 0
        mem.write(0x7ff, 0u8);

        Self { mem, rom_locked }
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        if address < PIF_RAM_START && self.rom_locked {
            panic!("Read from locked PIF ROM: {:08X}", address);
        }

        self.mem.read(address)
    }

    pub fn write<T: Size>(&mut self, joybus: &mut Joybus, address: u32, value: T) {
        if address < PIF_RAM_START {
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
            let mut ram = [0u8; 64];
            self.mem.read_be_bytes(PIF_RAM_START, &mut ram);
            joybus.execute(&mut ram);
            self.mem.write_be_bytes(PIF_RAM_START, &ram);
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
