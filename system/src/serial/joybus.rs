use arrayvec::ArrayVec;
use tracing::{debug, warn};

pub struct Joybus {
    joypads: [[u8; 4]; 4],
}

impl Joybus {
    pub fn new() -> Self {
        Self {
            joypads: [[0; 4]; 4],
        }
    }

    pub fn execute(&mut self, data: &mut [u8]) {
        debug!("PIF Joybus Input: {:X?}", data);

        let mut channel = 0;
        let mut index = 0;

        while index < (data.len() - 1) {
            let send_bytes = data[index] as usize;
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

            let recv_bytes = data[index] as usize;
            index += 1;

            if recv_bytes == 0xfe {
                break;
            }

            if (index + send_bytes) > data.len() {
                warn!("Joybus read overflow");
                break;
            }

            let send_data =
                ArrayVec::<u8, 64>::try_from(&data[index..(index + send_bytes)]).unwrap();

            index += send_bytes;

            if (index + recv_bytes) > data.len() {
                warn!("Joybus write overflow");
                break;
            }

            if let Some(recv_data) = self.perform_query(channel, &send_data) {
                let len = recv_data.len();

                if len != recv_bytes {
                    warn!("Received data does not match expected length. Expected {} bytes but got {} bytes.", recv_bytes, len);
                }

                data[index..(index + len)].copy_from_slice(&recv_data);
                index += len;
            } else {
                data[index - 2] |= 0x80;
            }

            channel += 1;
        }

        debug!("PIF Joybus Output: {:X?}", data);
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
                        // Provide 4 Kbit EEPROM by default
                        // TODO: Support other EEPROM sizes
                        output.push(0x00);
                        output.push(0x80);
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
