use audio::AudioInterface;
use cpu::{Cpu, Size};
use memory::{Mapping, Memory};
use mips_interface::MipsInterface;
use peripheral::PeripheralInterface;
use pif::Pif;
use rdp::Rdp;
use rdram::Rdram;
use rsp::Rsp;
use serial::SerialInterface;
use tracing::warn;
use video::VideoInterface;

mod audio;
mod cpu;
mod memory;
mod mips_interface;
mod peripheral;
mod pif;
mod rdp;
mod rdram;
mod rsp;
mod serial;
mod video;

struct Bus {
    memory_map: Vec<Mapping>,
    rdram: Rdram,
    rsp: Rsp,
    rdp: Rdp,
    mi: MipsInterface,
    vi: VideoInterface,
    ai: AudioInterface,
    pi: PeripheralInterface,
    si: SerialInterface,
    rom: Memory,
    pif: Pif,
}

pub struct Device {
    cpu: Cpu,
    bus: Bus,
    extra_cycle: bool,
}

impl Device {
    pub fn new(pif_data: Vec<u8>, rom_data: Vec<u8>) -> Self {
        let mut memory_map = vec![Mapping::None; 512];

        memory_map[0x03f] = Mapping::RdramRegister;
        memory_map[0x040] = Mapping::Rsp;
        memory_map[0x041] = Mapping::RdpCommand;
        memory_map[0x042] = Mapping::RdpSpan;
        memory_map[0x043] = Mapping::MipsInterface;
        memory_map[0x044] = Mapping::VideoInterface;
        memory_map[0x045] = Mapping::AudioInterface;
        memory_map[0x046] = Mapping::PeripheralInterface;
        memory_map[0x047] = Mapping::RdramInterface;
        memory_map[0x048] = Mapping::SerialInterface;
        memory_map[0x100..=0x1fb].fill(Mapping::CartridgeRom);
        memory_map[0x1fc] = Mapping::Pif;

        Self {
            cpu: Cpu::new(),
            bus: Bus {
                memory_map,
                rdram: Rdram::new(),
                rsp: Rsp::new(),
                rdp: Rdp::new(),
                mi: MipsInterface::new(),
                vi: VideoInterface::new(),
                ai: AudioInterface::new(),
                pi: PeripheralInterface::new(),
                si: SerialInterface::new(),
                rom: rom_data.into(),
                pif: Pif::new(pif_data),
            },
            extra_cycle: true,
        }
    }

    pub fn step(&mut self) {
        self.cpu.step(&mut self.bus);

        if self.extra_cycle {
            self.cpu.step(&mut self.bus);
        }

        self.bus.pi.step(&mut self.bus.rdram, &mut self.bus.rom);
    }
}

impl cpu::Bus for Bus {
    fn read_single<T: Size>(&self, address: u32) -> T {
        match self.memory_map[address as usize >> 20] {
            Mapping::RdramData => self.rdram.read_single(address),
            Mapping::RdramRegister => self.rdram.read_register(&self.mi, address & 0x000f_ffff),
            Mapping::Rsp => self.rsp.read(address & 0x000f_ffff),
            Mapping::RdpCommand => self.rdp.read_command(address & 0x000f_ffff),
            Mapping::RdpSpan => self.rdp.read_span(address & 0x000f_ffff),
            Mapping::MipsInterface => self.mi.read(address & 0x000f_ffff),
            Mapping::VideoInterface => self.vi.read(address & 0x000f_ffff),
            Mapping::AudioInterface => self.ai.read(address & 0x000f_ffff),
            Mapping::PeripheralInterface => self.pi.read(address & 0x000f_ffff),
            Mapping::RdramInterface => self.rdram.read_interface(address & 0x000f_ffff),
            Mapping::SerialInterface => self.si.read(address & 0x000f_ffff),
            Mapping::CartridgeRom => self.rom.read(address & 0x0fff_ffff),
            Mapping::Pif => self.pif.read(address & 0x000f_ffff),
            Mapping::None => {
                warn!("Unmapped read: {:08X}", address);
                T::zeroed()
            }
        }
    }

    fn write_single<T: Size>(&mut self, address: u32, value: T) {
        match self.memory_map[address as usize >> 20] {
            Mapping::RdramData => self.rdram.write_single(address, value),
            Mapping::RdramRegister => {
                self.rdram.write_register(
                    &mut self.mi,
                    &mut self.memory_map,
                    address & 0x000f_ffff,
                    value,
                );
            }
            Mapping::Rsp => self.rsp.write(address & 0x000f_ffff, value),
            Mapping::RdpCommand => self.rdp.write_command(address & 0x000f_ffff, value),
            Mapping::RdpSpan => self.rdp.write_span(address & 0x000f_ffff, value),
            Mapping::MipsInterface => self.mi.write(address & 0x000f_ffff, value),
            Mapping::VideoInterface => self.vi.write(address & 0x000f_ffff, value),
            Mapping::AudioInterface => self.ai.write(address & 0x000f_ffff, value),
            Mapping::PeripheralInterface => self.pi.write(address & 0x000f_ffff, value),
            Mapping::RdramInterface => self.rdram.write_interface(address & 0x000f_ffff, value),
            Mapping::SerialInterface => self.si.write(address & 0x000f_ffff, value),
            Mapping::CartridgeRom => panic!("Write to Cartridge ROM: {:08X}", address),
            Mapping::Pif => self.pif.write(address & 0x000f_ffff, value),
            Mapping::None => warn!("Unmapped write: {:08X}", address),
        }
    }

    fn read_block(&self, address: u32, data: &mut [u32]) {
        if self.memory_map[address as usize >> 20] != Mapping::RdramData {
            panic!("Only RDRAM data is supported for block reads");
        }

        self.rdram.read_block(address, data);
    }
}
