use crate::memory::{Mapping, Memory, Size, WriteMask};
use crate::mips_interface::MipsInterface;
use regs::{Delay, Mode, RasInterval, RefRow, RiConfig, RiMode, RiRefresh, RiSelect};
use std::array;
use tracing::{debug, warn};

mod regs;

const BANK_SIZE: usize = 1048576;

#[derive(Default)]
struct Module {
    device_id: u32,
    delay: Delay,
    mode: Mode,
    ref_row: RefRow,
    ras_interval: RasInterval,
}

#[derive(Default)]
struct Interface {
    mode: RiMode,
    config: RiConfig,
    select: RiSelect,
    refresh: RiRefresh,
}

pub struct Rdram {
    banks: [usize; 8],
    mem: Memory<u64>,
    modules: Vec<Module>,
    ri: Interface,
}

impl Rdram {
    pub fn new() -> Self {
        Self {
            // Default bank configuration (to support ROMs that use simplified
            // booting sequences)
            banks: array::from_fn(|index| (index * BANK_SIZE)),
            mem: Memory::with_byte_len(8 * BANK_SIZE),
            modules: (0..4)
                .map(|_| Module {
                    device_id: 0xffff,
                    ..Module::default()
                })
                .collect(),
            // TODO: Remove these default values if/when we eventually get RAM detection working
            ri: Interface {
                mode: 0x0e.into(),
                config: 0x40.into(),
                select: 0x14.into(),
                refresh: 0x0006_3634.into(),
            },
        }
    }

    pub fn read_single<T: Size>(&self, address: usize) -> T {
        let Some(bank_offset) = self.banks.get(address >> 20) else {
            warn!("Read outside RDRAM range: {:08X}", address);
            return T::zeroed();
        };

        let mapped_address = bank_offset + (address & 0x000f_ffff);
        self.mem.read(mapped_address)
    }

    pub fn write_single<T: Size>(&mut self, address: usize, value: T) {
        let Some(bank_offset) = self.banks.get(address >> 20) else {
            warn!("Write outside RDRAM range: {:08X}", address);
            return;
        };

        let mapped_address = bank_offset + (address & 0x000f_ffff);
        self.mem.write(mapped_address, value);
    }

    pub fn read_block<T: Size>(&self, address: usize, data: &mut [T]) {
        let Some(bank_offset) = self.banks.get(address >> 20) else {
            warn!("Read outside RDRAM range: {:08X}", address);
            data.fill(T::zeroed());
            return;
        };

        let mapped_address = bank_offset + (address & 0x000f_ffff);
        // TODO: What happens if we cross a non-contiguous bank boundary?
        self.mem.read_block(mapped_address, data);
    }

    pub fn write_block<T: Size>(&mut self, address: usize, data: &[T]) {
        let Some(bank_offset) = self.banks.get(address >> 20) else {
            warn!("Write outside RDRAM range: {:08X}", address);
            return;
        };

        let mapped_address = bank_offset + (address & 0x000f_ffff);
        // TODO: What happens if we cross a non-contiguous bank boundary?
        self.mem.write_block(mapped_address, data);
    }

    pub fn read_register<T: Size>(&self, mi: &MipsInterface, address: u32) -> T {
        // Broadcast mode
        if (address & 0x0008_0000) != 0 {
            panic!("Cannot broadcast a read");
        }

        // Single module mode
        let device_id = (address >> 10) & 0x01ff;

        for (index, module) in self.modules.iter().enumerate() {
            // Assume all modules are 2Mbit
            if (module.device_id & !1) == device_id {
                return T::truncate_u32(self.read_module_register(mi, index, address));
            }
        }

        panic!("Nothing responded to device ID {:04X}", device_id);
    }

    pub fn write_register<T: Size>(
        &mut self,
        mi: &mut MipsInterface,
        memory_map: &mut [Mapping],
        address: u32,
        value: T,
    ) {
        let mask = WriteMask::new(address, value);

        // Broadcast mode
        if (address & 0x0008_0000) != 0 {
            for index in 0..self.modules.len() {
                self.write_module_register(mi, memory_map, index, address, mask.clone());
            }
        } else {
            // Single module mode
            let device_id = (address >> 10) & 0x01ff;

            'outer: {
                for (index, module) in self.modules.iter().enumerate() {
                    // Assume all modules are 2Mbit
                    if (module.device_id & !1) == device_id {
                        self.write_module_register(mi, memory_map, index, address, mask);
                        break 'outer;
                    }
                }

                warn!("Nothing responded to device ID {:04X}", device_id);
            }
        }

        mi.clear_repeat()
    }

    pub fn read_interface<T: Size>(&self, address: u32) -> T {
        T::truncate_u32(match address >> 2 {
            0 => self.ri.mode.into(),
            3 => self.ri.select.into(),
            4 => self.ri.refresh.into(),
            _ => todo!("RI Register Read: {:08X}", address),
        })
    }

    pub fn write_interface<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        match address >> 2 {
            0 => {
                mask.write(&mut self.ri.mode);
                debug!("RI_MODE: {:?}", self.ri.mode);
            }
            1 => {
                mask.write(&mut self.ri.config);
                debug!("RI_CONFIG: {:?}", self.ri.config);
            }
            2 => {
                // This is a NOP as it's not real hardware...
                debug!("RI_CURRENT_LOAD complete");
            }
            3 => {
                mask.write(&mut self.ri.select);
                debug!("RI_SELECT: {:?}", self.ri.select);
                assert_eq!(0b0100, self.ri.select.rsel());
                assert_eq!(0b0001, self.ri.select.tsel());
            }
            4 => {
                mask.write(&mut self.ri.refresh);
                debug!("RI_REFRESH: {:?}", self.ri.refresh);
            }
            _ => todo!("RI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }

    fn read_module_register(&self, mi: &MipsInterface, index: usize, address: u32) -> u32 {
        let module = &self.modules[index];

        match (address & 0x03ff) >> 2 {
            0 => 0xb419_0010,
            3 => {
                assert!(mi.is_upper());
                u32::from(module.mode) ^ 0x40c0c0c0
            }
            9 => {
                assert!(mi.is_upper());
                0x0000_0200
            }
            _ => todo!("RDRAM{} Register Read: {:08X}", index, address,),
        }
    }

    fn write_module_register(
        &mut self,
        mi: &MipsInterface,
        memory_map: &mut [Mapping],
        index: usize,
        address: u32,
        mask: WriteMask,
    ) {
        let module = &mut self.modules[index];

        let mask = match module.delay.write_delay() {
            1 => mask,
            4 => {
                assert!(mi.is_repeat());
                mask.rotate(16)
            }
            delay => panic!("Unsupported write delay: {}", delay),
        };

        match (address & 0x03ff) >> 2 {
            1 => {
                let mut device_id = (module.device_id << 26)
                    | ((module.device_id << 17) & 0x0080_0000)
                    | ((module.device_id << 1) & 0xff00)
                    | ((module.device_id >> 8) & 0x0080);

                mask.write(&mut device_id);

                module.device_id = ((device_id & 0xfc00_0000) >> 26)
                    | ((device_id & 0x0080_0000) >> 17)
                    | ((device_id & 0xff00) >> 1)
                    | ((device_id & 0x0080) >> 8);

                debug!("RDRAM{} Device ID: {:04X}", index, module.device_id);

                self.remap(memory_map);
            }
            2 => {
                mask.write(&mut module.delay);
                debug!("RDRAM{} Delay: {:?}", index, module.delay);
                assert_eq!(1, module.delay.write_delay());
                assert_eq!(3, module.delay.ack_delay());
                assert_eq!(7, module.delay.read_delay());
                assert_eq!(5, module.delay.ack_win_delay());
            }
            3 => {
                mask.write(&mut module.mode);
                debug!("RDRAM{} Mode: {:?}", index, module.mode);
            }
            5 => {
                mask.write(&mut module.ref_row);
                debug!("RDRAM{} RefRow: {:?}", index, module.ref_row);
            }
            6 => {
                mask.write(&mut module.ras_interval);
                debug!("RDRAM{} RasInterval: {:?}", index, module.ras_interval);
            }
            _ => todo!(
                "RDRAM{} Register Write: {:08X} <= {:08x}",
                index,
                address,
                mask.raw()
            ),
        }
    }

    fn remap(&mut self, _memory_map: &mut [Mapping]) {
        // TODO: Re-enable
        // let mut bank_active = [false; 8];

        // // Assume 2MiB modules
        // for module in self.modules.iter().rev() {
        //     let device_id = (module.device_id & !1) as usize;

        //     if device_id >= 8 {
        //         continue;
        //     }

        //     let memory_start = device_id * BANK_SIZE;
        //     self.banks[device_id].offset = memory_start as u32;
        //     self.banks[device_id | 1].offset = (memory_start + BANK_SIZE) as u32;
        //     bank_active[device_id] = true;
        //     bank_active[device_id | 1] = true;
        // }

        // for (index, &active) in bank_active.iter().enumerate() {
        //     memory_map[index] = if active {
        //         debug!("Bank {}: {}", index, self.banks[index].offset);
        //         Mapping::RdramData
        //     } else {
        //         debug!("Bank {}: Unmapped", index);
        //         Mapping::None
        //     }
        // }
    }
}
