use super::Regs;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ExceptionStage {
    DC,
    EX,
    RF,
}

pub struct ExceptionDetails {
    pub code: u32,
    pub vector: u32,
    pub error: bool,
    pub ce: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Exception {
    Interrupt,
    TlbModification,
    TlbMissLoad,
    TlbMissStore,
    Syscall,
    Breakpoint,
    ReservedInstruction(u32),
    CoprocessorUnusable(u32),
}

impl Exception {
    pub fn process(self, _regs: &mut Regs) -> ExceptionDetails {
        match self {
            Exception::Interrupt => ExceptionDetails {
                code: 0,
                vector: 0x0180,
                error: false,
                ce: 0,
            },
            Exception::TlbModification => ExceptionDetails {
                code: 1,
                vector: 0x0180,
                error: false,
                ce: 0,
            },
            Exception::TlbMissLoad => ExceptionDetails {
                code: 2,
                vector: 0x0000,
                error: false,
                ce: 0,
            },
            Exception::TlbMissStore => ExceptionDetails {
                code: 3,
                vector: 0x0000,
                error: false,
                ce: 0,
            },
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
        }
    }
}
