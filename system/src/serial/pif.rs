use crate::memory::{Memory, Size};
use std::mem;
use tracing::{trace, warn};

const PIF_DATA_SIZE: usize = 2048;
const PIF_RAM_START: u32 = 0x7c0;

pub struct Pif {
    mem: Memory<u64>,
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

        // The initial command byte is always set to 0
        mem.write(0x7ff, 0u8);

        Self { mem, rom_locked }
    }

    pub fn ram(&self) -> &[u8] {
        &self.mem[PIF_RAM_START as usize..]
    }

    pub fn ram_mut(&mut self) -> &mut [u8] {
        &mut self.mem[PIF_RAM_START as usize..]
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        if address < PIF_RAM_START && self.rom_locked {
            panic!("Read from locked PIF ROM: {:08X}", address);
        }

        self.mem.read(address as usize)
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) -> bool {
        if address < PIF_RAM_START {
            panic!("Write to PIF ROM: {:08X}", address);
        }

        self.mem.write(address as usize, value);

        // Check if command byte was written
        if (address as usize + mem::size_of::<T>()) <= 0x7ff {
            return false;
        }

        // Interpret PIF command
        let cmd = self.mem.read::<u8>(0x7ff) & 0x7b;

        if cmd == 0 {
            return false;
        }

        let mut result: u8 = 0;

        let joybus_configure = (cmd & 0x01) != 0;

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

        joybus_configure
    }
}
