use audio::AudioInterface;
use cpu::{Cpu, Size};
use memory::Mapping;
use peripheral::PeripheralInterface;
use pif::Pif;
use rsp::Rsp;
use video::VideoInterface;

mod audio;
mod cpu;
mod memory;
mod peripheral;
mod pif;
mod rsp;
mod video;

struct Bus {
    memory_map: Vec<Mapping>,
    rsp: Rsp,
    vi: VideoInterface,
    ai: AudioInterface,
    pi: PeripheralInterface,
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
        memory_map[0x044] = Mapping::VideoInterface;
        memory_map[0x045] = Mapping::AudioInterface;
        memory_map[0x046] = Mapping::PeripheralInterface;
        memory_map[0x1fc] = Mapping::Pif;

        Self {
            cpu: Cpu::new(),
            bus: Bus {
                memory_map,
                rsp: Rsp::new(),
                vi: VideoInterface::new(),
                ai: AudioInterface::new(),
                pi: PeripheralInterface::new(),
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
            Mapping::VideoInterface => self.vi.read(address & 0x000f_ffff),
            Mapping::AudioInterface => self.ai.read(address & 0x000f_ffff),
            Mapping::PeripheralInterface => self.pi.read(address & 0x000f_ffff),
            Mapping::Pif => self.pif.read(address & 0x000f_ffff),
            Mapping::None => panic!("Unmapped read: {:08X}", address),
        }
    }

    fn write_single<T: Size>(&mut self, address: u32, value: T) {
        match self.memory_map[address as usize >> 20] {
            Mapping::Rsp => self.rsp.write(address & 0x000f_ffff, value),
            Mapping::VideoInterface => self.vi.write(address & 0x000f_ffff, value),
            Mapping::AudioInterface => self.ai.write(address & 0x000f_ffff, value),
            Mapping::PeripheralInterface => self.pi.write(address & 0x000f_ffff, value),
            Mapping::Pif => todo!("PIF writes"),
            Mapping::None => panic!("Unmapped write: {:08X}", address),
        }
    }
}
