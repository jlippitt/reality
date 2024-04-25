pub use audio::AudioReceiver;
pub use gfx::DisplayTarget;
pub use serial::JoypadState;

use audio::AudioInterface;
use cpu::{Cpu, Stats as CpuStats};
use gfx::GfxContext;
use interrupt::{CpuInterrupt, RcpInterrupt};
use memory::{Mapping, Memory, Size};
use mips_interface::MipsInterface;
use peripheral::PeripheralInterface;
use rdp::Rdp;
use rdram::Rdram;
use rsp::{Rsp, Stats as RspStats};
use serial::SerialInterface;
use std::error::Error;
use tracing::warn;
use video::VideoInterface;

mod audio;
mod cpu;
mod gfx;
mod interrupt;
mod memory;
mod mips_interface;
mod peripheral;
mod rdp;
mod rdram;
mod rsp;
mod serial;
mod video;

const RCP_CLOCK_RATE: f64 = 62500000.0;

const VIDEO_DAC_RATE: f64 = 1000000.0 * (18.0 * 227.5 / 286.0) * 17.0 / 5.0;

struct Bus {
    memory_map: Vec<Mapping>,
    cpu_int: CpuInterrupt,
    rdram: Rdram,
    rsp: Rsp,
    rdp: Rdp,
    mi: MipsInterface,
    vi: VideoInterface,
    ai: AudioInterface,
    pi: PeripheralInterface,
    si: SerialInterface,
    systest_buffer: Memory<u64>,
}

pub struct DeviceOptions<T: wgpu::WindowHandle + 'static> {
    pub display_target: DisplayTarget<T>,
    pub pif_data: Option<Vec<u8>>,
    pub rom_data: Vec<u8>,
}

#[cfg(feature = "profiling")]
pub struct Stats {
    pub cpu: CpuStats,
    pub rsp: RspStats,
}

pub struct Device {
    cpu: Cpu,
    bus: Bus,
    gfx: GfxContext,
    cycles: u64,
}

impl Device {
    pub fn new(options: DeviceOptions<impl wgpu::WindowHandle>) -> Result<Self, Box<dyn Error>> {
        let gfx = GfxContext::new(options.display_target)?;

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
        memory_map[0x050..=0x05f].fill(Mapping::DDRegisters);
        memory_map[0x100..=0x1fb].fill(Mapping::CartridgeRom);
        memory_map[0x1fc] = Mapping::Pif;

        // Default RDRAM mapping (for ROMs that use simplified boot sequences)
        memory_map[0x000..=0x007].fill(Mapping::RdramData);

        let cpu_int = CpuInterrupt::new();
        let rcp_int = RcpInterrupt::new(cpu_int.clone());

        let skip_pif_rom = options.pif_data.is_none();
        let ipl3_data = skip_pif_rom.then(|| &options.rom_data[0..0x1000]);

        // Detect CIC variant and populate initial values in PIF RAM and RDRAM
        let mut rdram = Rdram::new();
        let mut si = SerialInterface::new(rcp_int.clone(), options.pif_data);
        si.cic_detect(&options.rom_data, &mut rdram);

        Ok(Self {
            cpu: Cpu::new(skip_pif_rom),
            bus: Bus {
                memory_map,
                cpu_int,
                rdram,
                rsp: Rsp::new(rcp_int.clone(), ipl3_data),
                rdp: Rdp::new(rcp_int.clone(), &gfx),
                mi: MipsInterface::new(rcp_int.clone()),
                vi: VideoInterface::new(rcp_int.clone(), &gfx, skip_pif_rom)?,
                ai: AudioInterface::new(rcp_int.clone()),
                pi: PeripheralInterface::new(rcp_int, options.rom_data, skip_pif_rom),
                si,
                systest_buffer: Memory::with_byte_len(512),
            },
            gfx,
            cycles: 0,
        })
    }

    pub fn cycles(&self) -> u64 {
        self.cycles
    }

    pub fn sample_rate(&self) -> u32 {
        self.bus.ai.sample_rate()
    }

    #[cfg(feature = "profiling")]
    pub fn stats(&self) -> Stats {
        Stats {
            cpu: self.cpu.stats().clone(),
            rsp: self.bus.rsp.stats().clone(),
        }
    }

    #[cfg(feature = "profiling")]
    pub fn reset_stats(&mut self) {
        self.cpu.reset_stats();
        self.bus.rsp.reset_stats();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.gfx.resize(width, height);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        //self.bus.rdp.sync(&self.gfx, &mut self.bus.rdram);
        self.bus.vi.render(&self.bus.rdram, &self.gfx)
    }

    pub fn update_joypads(&mut self, joypads: &[JoypadState; 4]) {
        self.bus.si.update_joypads(joypads);
    }

    pub fn step(&mut self, receiver: &mut impl AudioReceiver) -> bool {
        self.cycles += 1;

        self.cpu.step(&mut self.bus);

        if (self.cycles & 1) == 0 {
            self.cpu.step(&mut self.bus);
        }

        self.bus.rsp.step_core(self.bus.rdp.shared());
        self.bus.rsp.step_dma(&mut self.bus.rdram);

        self.bus.rdp.step_core(&mut self.bus.rdram, &self.gfx);
        self.bus.rdp.step_dma(&self.bus.rdram, self.bus.rsp.mem());

        self.bus.ai.step(&self.bus.rdram, receiver);
        self.bus.pi.step(&mut self.bus.rdram);
        self.bus.si.step(&mut self.bus.rdram);
        self.bus.vi.step()
    }
}

impl cpu::Bus for Bus {
    fn read_single<T: Size>(&self, address: u32) -> T {
        match self.memory_map[address as usize >> 20] {
            Mapping::RdramData => self.rdram.read_single(address as usize),
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
            Mapping::DDRegisters => T::max_value(),
            Mapping::CartridgeRom => self.pi.read_rom(address & 0x0fff_ffff),
            Mapping::Pif => self.si.read_pif(address & 0x000f_ffff),
            Mapping::None => {
                warn!("Unmapped read: {:08X}", address);
                T::zeroed()
            }
        }
    }

    fn write_single<T: Size>(&mut self, address: u32, value: T) {
        match self.memory_map[address as usize >> 20] {
            Mapping::RdramData => self.rdram.write_single(address as usize, value),
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
            Mapping::DDRegisters => (), // Ignore
            Mapping::CartridgeRom => match address {
                0x13ff_0020..=0x13ff_0220 => {
                    self.systest_buffer
                        .write(address as usize - 0x13ff_0020, value);
                }
                0x13ff_0014 => println!(
                    "{}",
                    String::from_utf8_lossy(&self.systest_buffer[0..value.to_usize().unwrap()])
                ),
                _ => warn!("Write to Cartridge ROM: {:08X}", address),
            },
            Mapping::Pif => self.si.write_pif(address & 0x000f_ffff, value),
            Mapping::None => warn!("Unmapped write: {:08X}", address),
        }
    }

    fn read_block<T: Size>(&self, address: u32, data: &mut [T]) {
        if self.memory_map[address as usize >> 20] != Mapping::RdramData {
            panic!("Only RDRAM data is supported for block reads");
        }

        self.rdram.read_block(address as usize, data);
    }

    fn write_block<T: Size>(&mut self, address: u32, data: &[T]) {
        if self.memory_map[address as usize >> 20] != Mapping::RdramData {
            panic!("Only RDRAM data is supported for block writes");
        }

        self.rdram.write_block(address as usize, data);
    }

    fn poll(&self) -> u8 {
        self.cpu_int.status().bits()
    }
}
