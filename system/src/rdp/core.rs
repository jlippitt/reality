use super::renderer::Renderer;
use crate::gfx::GfxContext;
use crate::rdram::Rdram;
use std::collections::VecDeque;

mod image;
mod mode;
mod param;
mod rect;
mod sync;

pub struct Bus<'a> {
    pub renderer: &'a mut Renderer,
    pub rdram: &'a mut Rdram,
    pub gfx: &'a GfxContext,
}

pub struct Core {
    running: bool,
    commands: VecDeque<u64>,
}

impl Core {
    pub fn new() -> Self {
        Self {
            running: false,
            commands: VecDeque::new(),
        }
    }

    pub fn running(&self) -> bool {
        self.running
    }

    pub fn restart(&mut self) {
        self.running = true;
    }

    pub fn write_command(&mut self, value: u64) {
        self.commands.push_back(value)
    }

    pub fn step(&mut self, bus: Bus) -> bool {
        let Some(word) = self.commands.pop_front() else {
            self.running = false;
            return false;
        };

        let opcode = (word >> 56) & 0x3f;

        match opcode {
            0x29 => sync::sync_full(self, bus, word),
            0x2d => image::set_scissor(self, bus, word),
            0x2f => mode::set_other_modes(self, bus, word),
            0x36 => rect::fill_rectangle(self, bus, word),
            0x37 => param::set_fill_color(self, bus, word),
            0x3f => image::set_color_image(self, bus, word),
            _ => todo!("RDP Command: {:#02X}", opcode),
        }

        // If SYNC_FULL was run, let the caller know
        opcode == 0x29
    }
}
