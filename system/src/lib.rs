use cpu::{Cpu, Size};
use memory::Mapping;
use pif::Pif;
use rsp::Rsp;

mod cpu;
mod memory;
mod pif;
mod rsp;

struct Bus {
    memory_map: Vec<Mapping>,
    rsp: Rsp,
    pif: Pif,
}

pub struct Device {
    cpu: Cpu,
    bus: Bus,
}

impl Device {
    pub fn new(pif_data: Vec<u8>) -> Self {
        let mut memory_map = vec![Mapping::None; 512];

        memory_map[0x040] = Mapping::Rsp;
        memory_map[0x1fc] = Mapping::Pif;

        Self {
            cpu: Cpu::new(),
            bus: Bus {
                memory_map,
                rsp: Rsp::new(),
                pif: Pif::new(pif_data),
            },
        }
    }

    pub fn step(&mut self) {
        self.cpu.step(&mut self.bus);
    }
}

impl cpu::Bus for Bus {
    fn read_single<T: Size>(&self, address: u32) -> T {
        match self.memory_map[address as usize >> 20] {
            Mapping::Rsp => self.rsp.read(address & 0x000f_ffff),
            Mapping::Pif => self.pif.read(address & 0x000f_ffff),
            Mapping::None => T::zeroed(),
        }
    }

    fn write_single<T: Size>(&mut self, address: u32, value: T) {
        match self.memory_map[address as usize >> 20] {
            Mapping::Rsp => self.rsp.write(address & 0x000f_ffff, value),
            Mapping::Pif => todo!("PIF writes"),
            Mapping::None => (),
        }
    }
}
