use super::Regs;

pub struct ExceptionDetails {
    pub code: u32,
    pub vector: u32,
    pub error: bool,
    pub ce: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Exception {
    Interrupt,
    TlbModification(u32),
    TlbMissLoad(u32, bool),
    TlbMissStore(u32, bool),
    Syscall,
    Breakpoint,
    ReservedInstruction(u32),
    CoprocessorUnusable(u32),
    Trap,
}

impl Exception {
    pub fn process(self, regs: &mut Regs) -> ExceptionDetails {
        match self {
            Exception::Interrupt => ExceptionDetails {
                code: 0,
                vector: 0x0180,
                error: false,
                ce: 0,
            },
            Exception::TlbModification(vaddr) => {
                regs.context.set_bad_vpn2(vaddr >> 13);
                regs.bad_vaddr = vaddr;
                regs.entry_hi.set_vpn2(vaddr as u64 >> 13);
                regs.x_context.set_bad_vpn2(vaddr as u64 >> 13);

                ExceptionDetails {
                    code: 1,
                    vector: 0x0180,
                    error: false,
                    ce: 0,
                }
            }
            Exception::TlbMissLoad(vaddr, invalid) => {
                regs.context.set_bad_vpn2(vaddr >> 13);
                regs.bad_vaddr = vaddr;
                regs.entry_hi.set_vpn2(vaddr as u64 >> 13);
                regs.x_context.set_bad_vpn2(vaddr as u64 >> 13);

                ExceptionDetails {
                    code: 2,
                    vector: if invalid || regs.status.exl() {
                        0x0180
                    } else {
                        0x0000
                    },
                    error: false,
                    ce: 0,
                }
            }
            Exception::TlbMissStore(vaddr, invalid) => {
                regs.context.set_bad_vpn2(vaddr >> 13);
                regs.bad_vaddr = vaddr;
                regs.entry_hi.set_vpn2(vaddr as u64 >> 13);
                regs.x_context.set_bad_vpn2(vaddr as u64 >> 13);

                ExceptionDetails {
                    code: 3,
                    vector: if invalid || regs.status.exl() {
                        0x0180
                    } else {
                        0x0000
                    },
                    error: false,
                    ce: 0,
                }
            }
            Exception::Syscall => ExceptionDetails {
                code: 8,
                vector: 0x0180,
                error: false,
                ce: 0,
            },
            Exception::Breakpoint => ExceptionDetails {
                code: 9,
                vector: 0x0180,
                error: false,
                ce: 0,
            },
            Exception::ReservedInstruction(ce) => ExceptionDetails {
                code: 10,
                vector: 0x0180,
                error: false,
                ce,
            },
            Exception::CoprocessorUnusable(ce) => ExceptionDetails {
                code: 11,
                vector: 0x0180,
                error: false,
                ce,
            },
            Exception::Trap => ExceptionDetails {
                code: 13,
                vector: 0x0180,
                error: false,
                ce: 0,
            },
        }
    }
}
