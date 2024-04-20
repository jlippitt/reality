use tracing::trace;

#[derive(Debug)]
pub struct CombineModeRawParams {
    pub sub_a: u32,
    pub sub_b: u32,
    pub mul: u32,
    pub add: u32,
}

#[derive(Debug)]
pub struct CombineModeRaw {
    pub rgb: [CombineModeRawParams; 2],
    pub alpha: [CombineModeRawParams; 2],
}

pub struct Combiner {
    hash_value: u64,
}

impl Combiner {
    pub fn new() -> Self {
        Self { hash_value: 0 }
    }

    pub fn hash_value(&self) -> u64 {
        self.hash_value
    }

    pub fn set_combine_mode(&mut self, combine_mode: CombineModeRaw, hash_value: u64) {
        self.hash_value = hash_value;
        trace!("  Combiner Hash Value: {:08X}", self.hash_value);

        trace!("  = {:?}", combine_mode);
    }
}
