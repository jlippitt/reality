pub struct Cp0 {
    regs: [i64; 32],
}

impl Cp0 {
    pub const REG_NAMES: [&'static str; 32] = [
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

    pub fn new() -> Self {
        Self { regs: [0; 32] }
    }

    pub fn write_reg(&mut self, reg: usize, value: i64) {
        self.regs[reg] = value;

        match reg {
            _ => todo!("Write to {}", Self::REG_NAMES[reg]),
        }
    }
}
