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
    pub count: u32,
    pub compare: u32,
    pub status: Status,
    pub cause: Cause,
    pub epc: u32,
    pub config: Config,
    pub ll_addr: u32,
    pub tag_lo: TagLo,
    pub tag_hi: u32,
    pub error_epc: u32,
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
    pub fr: bool,
    pub rp: bool,
    pub cu0: bool,
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
    #[bits(3)]
    pub k0: u32,
    pub cu: bool,
    #[bits(11)]
    __: u32,
    pub be: bool,
    #[bits(8)]
    __: u32,
    #[bits(4)]
    pub ep: u32,
    #[bits(3)]
    pub ec: u32,
    __: bool,
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
