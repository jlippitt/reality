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
    pub stage: ExceptionStage,
    pub error: bool,
    pub ce: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Exception {
    Interrupt,
    Syscall,
    Breakpoint,
    CoprocessorUnusable(u32),
}

impl Exception {
    pub fn process(self, _regs: &mut Regs) -> ExceptionDetails {
        match self {
            Exception::Interrupt => ExceptionDetails {
                code: 0,
                vector: 0x0180,
                stage: ExceptionStage::DC,
                error: false,
                ce: 0,
            },
            Exception::Syscall => ExceptionDetails {
                code: 8,
                vector: 0x0180,
                stage: ExceptionStage::EX,
                error: false,
                ce: 0,
            },
            Exception::Breakpoint => ExceptionDetails {
                code: 9,
                vector: 0x0180,
                stage: ExceptionStage::EX,
                error: false,
                ce: 0,
            },
            Exception::CoprocessorUnusable(ce) => ExceptionDetails {
                code: 11,
                vector: 0x0180,
                stage: ExceptionStage::EX,
                error: false,
                ce,
            },
        }
    }
}
