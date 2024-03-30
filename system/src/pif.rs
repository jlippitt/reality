const PIF_DATA_SIZE: usize = 2048;

pub struct Pif {
    data: Vec<u8>,
}

impl Pif {
    pub fn new(data: Vec<u8>) -> Self {
        assert!(data.len() == PIF_DATA_SIZE);
        Self { data }
    }
}
