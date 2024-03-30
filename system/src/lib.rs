use pif::Pif;

mod pif;

#[derive(Copy, Clone, Debug)]
enum Mapping {
    None,
    Pif,
}

pub struct Device {
    memory_map: Vec<Mapping>,
    pif: Pif,
}

impl Device {
    pub fn new(pif_data: Vec<u8>) -> Self {
        let mut memory_map = vec![Mapping::None; 512];
        memory_map[0x1fc] = Mapping::Pif;
        Self {
            memory_map,
            pif: Pif::new(pif_data),
        }
    }
}
