#![allow(clippy::unnecessary_cast)]

use bitfield_struct::bitfield;

pub const REG_NAMES: [&str; 32] = [
    "Index",
    "Random",
    "EntryLo0",
    "EntryLo1",
    "Context",
    "PageMask",
    "Wired",
    "R7",
    "BadVAddr",
    "Count",
    "EntryHi",
    "Compare",
    "Status",
    "Cause",
    "EPC",
    "PRId",
    "Config",
    "LLAddr",
    "WatchLo",
    "WatchHi",
    "XContext",
    "R21",
    "R22",
    "R23",
    "R24",
    "R25",
    "ParityError",
    "CacheError",
    "TagLo",
    "TagHi",
    "ErrorEPC",
    "R31",
];

#[derive(Default, Debug)]
pub struct Regs {
    pub index: Index,
    pub entry_lo0: EntryLo,
    pub entry_lo1: EntryLo,
    pub context: Context,
    pub page_mask: PageMask,
    pub wired: Wired,
    pub bad_vaddr: u32,
    pub count: u32,
    pub entry_hi: EntryHi,
    pub compare: u32,
    pub status: Status,
    pub cause: Cause,
    pub epc: u32,
    pub config: Config,
    pub ll_addr: u32,
    pub watch_lo: WatchLo,
    pub watch_hi: WatchHi,
    pub x_context: XContext,
    pub tag_lo: TagLo,
    pub tag_hi: u32,
    pub error_epc: u32,
}

#[bitfield(u32)]
pub struct Index {
    #[bits(6)]
    pub index: u32,
    #[bits(25)]
    __: u32,
    pub probe_failure: bool,
}

#[bitfield(u32)]
pub struct EntryLo {
    pub global: bool,
    pub valid: bool,
    pub dirty: bool,
    #[bits(3)]
    pub cache: u32,
    #[bits(20)]
    pub pfn: u32,
    #[bits(6)]
    __: u32,
}

#[bitfield(u32)]
pub struct Context {
    #[bits(4)]
    __: u32,
    #[bits(19)]
    bad_vpn2: u32,
    #[bits(9)]
    pte_base: u32,
}

#[bitfield(u32)]
pub struct PageMask {
    #[bits(13)]
    __: u32,
    #[bits(12)]
    pub mask: u32,
    #[bits(7)]
    __: u32,
}

#[bitfield(u32)]
pub struct Wired {
    #[bits(6)]
    pub index: u32,
    #[bits(26)]
    __: u32,
}

#[bitfield(u32)]
pub struct EntryHi {
    #[bits(8)]
    pub asid: u32,
    #[bits(4)]
    __: u32,
    pub global: bool,
    #[bits(19)]
    pub vpn2: u32,
}

#[bitfield(u32)]
pub struct Status {
    pub ie: bool,
    pub exl: bool,
    pub erl: bool,
    #[bits(2)]
    pub ksu: u32,
    pub ux: bool,
    pub sx: bool,
    pub kx: bool,
    pub im: u8,
    #[bits(9)]
    pub ds: u32,
    pub re: bool,
    #[bits(default = true)]
    pub fr: bool,
    pub rp: bool,
    #[bits(default = true)]
    pub cu0: bool,
    #[bits(default = true)]
    pub cu1: bool,
    pub cu2: bool,
    pub cu3: bool,
}

#[bitfield(u32)]
pub struct Cause {
    #[bits(2)]
    __: u32,
    #[bits(5)]
    pub exc_code: u32,
    __: bool,
    pub ip: u8,
    #[bits(12)]
    __: u32,
    #[bits(2)]
    pub ce: u32,
    __: bool,
    pub bd: bool,
}

#[bitfield(u32)]
pub struct Config {
    #[bits(3, default = 0b011)]
    pub k0: u32,
    pub cu: bool,
    #[bits(11, default = 0b11001000110)]
    __: u32,
    #[bits(default = true)]
    pub be: bool,
    #[bits(8, default = 0b00000110)]
    __: u32,
    #[bits(4)]
    pub ep: u32,
    #[bits(3, access = RO, default = 0b111)]
    pub ec: u32,
    __: bool,
}

#[bitfield(u32)]
pub struct WatchLo {
    pub write: bool,
    pub read: bool,
    __: bool,
    #[bits(29)]
    pub paddr0: u32,
}

#[bitfield(u32)]
pub struct WatchHi {
    #[bits(4)]
    pub paddr1: u32,
    #[bits(28)]
    __: u32,
}

#[bitfield(u64)]
pub struct XContext {
    #[bits(4)]
    __: u64,
    #[bits(27)]
    bad_vpn2: u64,
    #[bits(2)]
    region: u64,
    #[bits(31)]
    pte_base: u64,
}

#[bitfield(u32)]
pub struct TagLo {
    #[bits(6)]
    __: u32,
    #[bits(2)]
    pub pstate: u32,
    #[bits(20)]
    pub ptag_lo: u32,
    #[bits(4)]
    __: u32,
}
