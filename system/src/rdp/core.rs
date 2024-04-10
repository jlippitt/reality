use super::renderer::Renderer;
use crate::gfx::GfxContext;
use crate::rdram::Rdram;
use bitfield_struct::bitfield;
use std::collections::VecDeque;
use tracing::trace;

mod mode;

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

    pub fn step(&mut self, bus: Bus) {
        let Some(word) = self.commands.pop_front() else {
            self.running = false;
            return;
        };

        match (word >> 56) & 0x3f {
            0x2d => set_scissor(self, bus, word),
            0x2f => mode::set_other_modes(self, bus, word),
            opcode => todo!("RDP Command: {:#02X}", opcode),
        }
    }
}

fn set_scissor(_core: &mut Core, _bus: Bus, word: u64) {
    let cmd = SetScissor::from(word);

    trace!("SET_SCISSOR {:?}", cmd);

    if cmd.field() {
        todo!("Set_Scissor interlace suppport");
    }

    // bus.renderer.set_scissor(cmd.xh(), cmd.yh(), cmd.xl(), cmd.yl());
}

#[bitfield(u64)]
struct SetScissor {
    #[bits(12)]
    yl: u64,
    #[bits(12)]
    xl: u64,
    odd_line: bool,
    field: bool,
    #[bits(6)]
    __: u64,
    #[bits(12)]
    yh: u64,
    #[bits(12)]
    xh: u64,
    #[bits(8)]
    __: u64,
}
