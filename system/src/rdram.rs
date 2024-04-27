use crate::memory::{Memory, Size, WriteMask};
use crate::mips_interface::MipsInterface;
use regs::{Delay, Mode, RasInterval, RefRow, RiConfig, RiMode, RiRefresh, RiSelect};
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
    mem: Memory<u64>,
    modules: Vec<Module>,
    ri: Interface,
}

impl Rdram {
    pub fn new() -> Self {
        Self {
            mem: Memory::with_byte_len(8 * BANK_SIZE),
            modules: (0..4)
                .map(|_| Module {
                    device_id: 0xffff,
                    ..Module::default()
                })
                .collect(),
            ri: Interface {
                mode: 0x0e.into(),
                config: 0x40.into(),
                select: 0x14.into(),
                refresh: 0x0006_3634.into(),
            },
        }
    }

    pub fn read_single<T: Size>(&self, address: usize) -> T {
        self.mem.read_or_zero(address)
    }

    pub fn write_single<T: Size>(&mut self, address: usize, value: T) {
        self.mem.write_or_ignore(address, value);
    }

    pub fn read_block<T: Size>(&self, address: usize, data: &mut [T]) {
        self.mem.read_or_zero_block(address, data);
    }

    pub fn write_block<T: Size>(&mut self, address: usize, data: &[T]) {
        self.mem.write_or_ignore_block(address, data);
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

    pub fn write_register<T: Size>(&mut self, mi: &mut MipsInterface, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        // Broadcast mode
        if (address & 0x0008_0000) != 0 {
            for index in 0..self.modules.len() {
                self.write_module_register(mi, index, address, mask.clone());
            }
        } else {
            // Single module mode
            let device_id = (address >> 10) & 0x01ff;

            'outer: {
                for (index, module) in self.modules.iter().enumerate() {
                    // Assume all modules are 2Mbit
                    if (module.device_id & !1) == device_id {
                        self.write_module_register(mi, index, address, mask);
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
}
