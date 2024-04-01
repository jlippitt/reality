use crate::cpu::Size;
use crate::memory::WriteMask;
use crate::mips_interface::MipsInterface;
use regs::{Delay, Mode, RefRow, RiConfig, RiMode, RiSelect};
use tracing::trace;

mod regs;

#[derive(Default)]
struct Module {
    device_id: u32,
    delay: Delay,
    mode: Mode,
    ref_row: RefRow,
}

struct Interface {
    mode: RiMode,
    config: RiConfig,
    select: RiSelect,
}

pub struct Rdram {
    ri: Interface,
    modules: Vec<Module>,
}

impl Rdram {
    pub fn new() -> Self {
        Self {
            ri: Interface {
                mode: RiMode::new(),
                config: RiConfig::new(),
                select: RiSelect::new(),
            },
            modules: vec![Module::default(), Module::default()],
        }
    }

    pub fn read_register<T: Size>(&self, address: u32) -> T {
        todo!("RDRAM Register Read: {:08X}", address);
    }

    pub fn write_register<T: Size>(&mut self, mi: &mut MipsInterface, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        // Broadcast mode
        if (address & 0x0008_0000) != 0 {
            for (index, module) in self.modules.iter_mut().enumerate() {
                module.write_register(mi, index, address, mask.clone());
            }
        } else {
            // Single module mode
            let device_id = (address >> 10) & 0x01ff;

            'outer: {
                for (index, module) in self.modules.iter_mut().enumerate() {
                    if module.device_id == device_id {
                        self.modules[index].write_register(mi, index, address, mask);
                        break 'outer;
                    }
                }

                panic!("Nothing responded to device ID {:04X}", device_id);
            }
        }

        mi.clear_repeat()
    }

    pub fn read_interface<T: Size>(&self, address: u32) -> T {
        T::from_u32(match address >> 2 {
            0 => self.ri.mode.into(),
            3 => self.ri.select.into(),
            _ => todo!("RI Register Read: {:08X}", address),
        })
    }

    pub fn write_interface<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        match address >> 2 {
            0 => {
                mask.write(&mut self.ri.mode);
                trace!("RI_MODE: {:?}", self.ri.mode);
            }
            1 => {
                mask.write(&mut self.ri.config);
                trace!("RI_CONFIG: {:?}", self.ri.config);
            }
            2 => {
                // This is a NOP as it's not real hardware...
                trace!("RI_CURRENT_LOAD complete");
            }
            3 => {
                mask.write(&mut self.ri.select);
                trace!("RI_SELECT: {:?}", self.ri.select);
                assert_eq!(0b0100, self.ri.select.rsel());
                assert_eq!(0b0001, self.ri.select.tsel());
            }
            _ => todo!("RI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }
}

impl Module {
    pub fn write_register(
        &mut self,
        mi: &MipsInterface,
        index: usize,
        address: u32,
        mask: WriteMask,
    ) {
        let mask = match self.delay.write_delay() {
            1 => mask,
            4 => {
                assert!(mi.is_repeat());
                mask.rotate(16)
            }
            delay => panic!("Unsupported write delay: {}", delay),
        };

        match (address & 0x03ff) >> 2 {
            1 => {
                let mut device_id = (self.device_id << 26)
                    | ((self.device_id << 17) & 0x0080_0000)
                    | ((self.device_id << 1) & 0xff00)
                    | ((self.device_id >> 8) & 0x0080);

                mask.write(&mut device_id);

                self.device_id = ((device_id & 0xfc00_0000) >> 26)
                    | ((device_id & 0x0080_0000) >> 17)
                    | ((device_id & 0xff00) >> 1)
                    | ((device_id & 0x0080) >> 8);

                trace!("RDRAM{} Device Id: {:04X}", index, self.device_id);
            }
            2 => {
                mask.write(&mut self.delay);
                trace!("RDRAM{} Delay: {:?}", index, self.delay);
                assert_eq!(1, self.delay.write_delay());
                assert_eq!(3, self.delay.ack_delay());
                assert_eq!(7, self.delay.read_delay());
                assert_eq!(5, self.delay.ack_win_delay());
            }
            3 => {
                mask.write(&mut self.mode);
                trace!("RDRAM{} Mode: {:?}", index, self.mode);
            }
            5 => {
                mask.write(&mut self.ref_row);
                trace!("RDRAM{} RefRow: {:?}", index, self.ref_row);
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
