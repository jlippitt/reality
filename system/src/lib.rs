use bytemuck::Pod;
use cpu::Cpu;
use memory::Mapping;
use pif::Pif;

mod cpu;
mod memory;
mod pif;

struct Bus {
    memory_map: Vec<Mapping>,
    pif: Pif,
}

pub struct Device {
    cpu: Cpu,
    bus: Bus,
}

impl Device {
    pub fn new(pif_data: Vec<u8>) -> Self {
        let mut memory_map = vec![Mapping::None; 512];

        memory_map[0x1fc] = Mapping::Pif;

        Self {
            cpu: Cpu::new(),
            bus: Bus {
                memory_map,
                pif: Pif::new(pif_data),
            },
        }
    }

    pub fn step(&mut self) {
        self.cpu.step(&mut self.bus);
    }
}

impl cpu::Bus for Bus {
    fn read_single<T: Pod>(&self, address: u32) -> T {
        match self.memory_map[address as usize >> 20] {
            Mapping::Pif => self.pif.read(address & 0x000f_ffff),
            Mapping::None => T::zeroed(),
        }
    }
}
