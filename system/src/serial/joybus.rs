use crate::header::SaveType;
use arrayvec::ArrayVec;
use tracing::{debug, trace, warn};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct JoypadState {
    pub a: bool,
    pub b: bool,
    pub c_up: bool,
    pub c_down: bool,
    pub c_left: bool,
    pub c_right: bool,
    pub l: bool,
    pub r: bool,
    pub z: bool,
    pub start: bool,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    pub axis_x: i8,
    pub axis_y: i8,
}

pub struct Joybus {
    program: [u8; 64],
    joypads: [[u8; 4]; 4],
    save_type: SaveType,
}

impl Joybus {
    pub fn new(save_type: SaveType) -> Self {
        Self {
            program: [0; 64],
            joypads: [[0; 4]; 4],
            save_type,
        }
    }

    pub fn update_joypads(&mut self, joypads: &[JoypadState; 4]) {
        for (dst, src) in self.joypads.iter_mut().zip(joypads.iter()) {
            dst[0] = 0;
            dst[0] |= if src.a { 0x80 } else { 0 };
            dst[0] |= if src.b { 0x40 } else { 0 };
            dst[0] |= if src.z { 0x20 } else { 0 };
            dst[0] |= if src.start { 0x10 } else { 0 };
            dst[0] |= if src.dpad_up { 0x08 } else { 0 };
            dst[0] |= if src.dpad_down { 0x04 } else { 0 };
            dst[0] |= if src.dpad_left { 0x02 } else { 0 };
            dst[0] |= if src.dpad_right { 0x01 } else { 0 };

            // RST 'button' possibly doesn't need to be implemented?
            dst[1] = 0;
            dst[1] |= if src.l { 0x20 } else { 0 };
            dst[1] |= if src.r { 0x10 } else { 0 };
            dst[1] |= if src.c_up { 0x08 } else { 0 };
            dst[1] |= if src.c_down { 0x04 } else { 0 };
            dst[1] |= if src.c_left { 0x02 } else { 0 };
            dst[1] |= if src.c_right { 0x01 } else { 0 };

            dst[2] = src.axis_x as u8;
            dst[3] = src.axis_y as u8;
        }
    }

    pub fn configure(&mut self, pif_ram: &[u8]) {
        self.program.copy_from_slice(pif_ram);
        trace!("Joybus Configured");
    }

    pub fn execute(&mut self, pif_ram: &mut [u8]) {
        debug!("PIF Joybus Input: {:X?}", self.program);

        let mut channel = 0;
        let mut index = 0;

        while index < (self.program.len() - 1) {
            let send_bytes = self.program[index] as usize;
            index += 1;

            if send_bytes == 0xfe {
                break;
            }

            if (send_bytes & 0xc0) != 0 {
                continue;
            }

            if send_bytes == 0 {
                channel += 1;
                continue;
            }

            let recv_bytes = self.program[index] as usize;
            index += 1;

            if recv_bytes == 0xfe {
                break;
            }

            if (index + send_bytes) > self.program.len() {
                warn!("Joybus read overflow");
                break;
            }

            let send_data =
                ArrayVec::<u8, 64>::try_from(&self.program[index..(index + send_bytes)]).unwrap();

            index += send_bytes;

            if (index + recv_bytes) > self.program.len() {
                warn!("Joybus write overflow");
                break;
            }

            if let Some(recv_data) = self.perform_query(channel, &send_data) {
                let len = recv_data.len();

                if len != recv_bytes {
                    warn!("Received data does not match expected length. Expected {} bytes but got {} bytes.", recv_bytes, len);
                }

                pif_ram[index..(index + len)].copy_from_slice(&recv_data);
                index += len;
            } else {
                pif_ram[index - 2] |= 0x80;
            }

            channel += 1;
        }

        debug!("PIF Joybus Output: {:X?}", pif_ram);
    }

    fn perform_query(&self, channel: usize, input: &[u8]) -> Option<ArrayVec<u8, 64>> {
        let mut output = ArrayVec::new();

        match input[0] {
            0x00 | 0xff => {
                match channel {
                    0 => {
                        output.push(0x05);
                        output.push(0x00);
                        output.push(0x02); // TODO: Controller Pak
                    }
                    1..=3 => return None,
                    4 => {
                        // TODO: This shouldn't return anything for non-EEPROM saves
                        output.push(0x00);

                        output.push(if self.save_type == SaveType::Eeprom16K {
                            0xc0
                        } else {
                            0x80
                        });

                        output.push(0x00); // TODO: 'Write in progress' flag
                    }
                    _ => panic!("Invalid JoyBus channel: {}", channel),
                }
            }
            0x01 => {
                if channel > 3 {
                    panic!("Invalid JoyBus channel: {}", channel);
                }

                output
                    .try_extend_from_slice(&self.joypads[channel])
                    .unwrap();
            }
            0x02 => {
                if channel > 3 {
                    panic!("Invalid JoyBus channel: {}", channel);
                }

                warn!("Controller Pak reads not yet implemented");

                for _ in 0..32 {
                    output.push(0);
                }

                output.push(calc_crc(&output[0..32]));
            }
            0x03 => {
                if channel > 3 {
                    panic!("Invalid JoyBus channel: {}", channel);
                }

                warn!("Controller Pak writes not yet implemented");
                output.push(calc_crc(&input[3..35]));
            }
            0x04 => {
                // TODO: EEPROM reads
                for _ in 0..8 {
                    output.push(0x00);
                }
            }
            0x05 => {
                // TODO: EEPROM writes
                // TODO: 'Write in progress' flag
                output.push(0x00);
            }
            _ => panic!("Unknown JoyBus command: {:02X}", input[0]),
        }

        Some(output)
    }
}

fn calc_crc(data: &[u8]) -> u8 {
    debug_assert!(data.len() == 32);

    let mut result: u8 = 0;

    for index in 0..=data.len() {
        for bit in (0..=7).rev() {
            let xor_tap = if (result & 0x80) != 0 { 0x85 } else { 0 };
            result <<= 1;

            if index < data.len() && (data[index] & (1 << bit)) != 0 {
                result |= 1;
            }

            result ^= xor_tap;
        }
    }

    result
}
