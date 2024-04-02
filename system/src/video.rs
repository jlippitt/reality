use super::memory::{Size, WriteMask};
use regs::Regs;
use tracing::warn;

mod regs;

pub struct VideoInterface {
    regs: Regs,
    h_counter: u32,
    v_counter: u32,
}

impl VideoInterface {
    pub fn new() -> Self {
        Self {
            regs: Regs::default(),
            v_counter: 0,
            h_counter: 0,
        }
    }

    pub fn step(&mut self) -> bool {
        let mut frame_done = false;
        let h_sync = self.regs.h_sync.h_sync();

        self.h_counter += 1;

        if self.h_counter >= h_sync {
            self.v_counter += 1;
            self.h_counter -= h_sync;

            if self.v_counter >= self.regs.v_sync.v_sync() {
                self.v_counter = 0;
                frame_done = true;
            }

            // TODO: Set V_CURRENT
            // TODO: VI interrupt
        }

        frame_done
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        todo!("VI Register Read: {:08X}", address);
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        match address >> 2 {
            0 => mask.write_reg("VI_CTRL", &mut self.regs.ctrl),
            1 => mask.write_reg_hex("VI_ORIGIN", &mut self.regs.origin),
            2 => mask.write_reg("VI_WIDTH", &mut self.regs.width),
            3 => mask.write_reg("VI_V_INTR", &mut self.regs.v_intr),
            4 => warn!("TODO: Acknowledge VI interrupt"),
            5 => mask.write_reg("VI_BURST", &mut self.regs.burst),
            6 => mask.write_reg("VI_V_SYNC", &mut self.regs.v_sync),
            7 => mask.write_reg("VI_H_SYNC", &mut self.regs.h_sync),
            8 => mask.write_reg("VI_H_SYNC_LEAP", &mut self.regs.h_sync_leap),
            9 => mask.write_reg("VI_H_VIDEO", &mut self.regs.h_video),
            10 => mask.write_reg("VI_V_VIDEO", &mut self.regs.v_video),
            11 => mask.write_reg("VI_V_BURST", &mut self.regs.v_burst),
            12 => mask.write_reg("VI_X_SCALE", &mut self.regs.x_scale),
            13 => mask.write_reg("VI_Y_SCALE", &mut self.regs.y_scale),
            14 => mask.write_reg("VI_TEST_ADDR", &mut self.regs.test_addr),
            15 => mask.write_reg("VI_STAGED_DATA", &mut self.regs.staged_data),
            _ => todo!("VI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }
}
