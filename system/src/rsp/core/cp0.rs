pub use ex::cop0;

use super::{Core, DfState};

mod ex;

const REG_NAMES: [&str; 16] = [
    "SP_DMA_SPADDR",
    "SP_DMA_RAMADDR",
    "SP_DMA_RDLEN",
    "SP_DMA_WRLEN",
    "SP_STATUS",
    "SP_DMA_FULL",
    "SP_DMA_BUSY",
    "SP_SEMAPHORE",
    "DPC_START",
    "DPC_END",
    "DPC_CURRENT",
    "DPC_STATUS",
    "DPC_CLOCK",
    "DPC_BUF_BUSY",
    "DPC_PIPE_BUSY",
    "DPC_TMEM_BUSY",
];
