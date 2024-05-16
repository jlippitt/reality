use crc::Crc;
use phf::{phf_map, Map};
use std::fmt::{self, Display, Formatter};
use tracing::debug;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum CicType {
    Unknown,
    Nus6101,
    Nus6102,
    Nus6103,
    Nus6105,
    Nus6106,
    MiniIPL3,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SaveType {
    Eeprom4K,
    Eeprom16K,
}

pub struct Header {
    pub cic_type: CicType,
    pub save_type: SaveType,
}

pub fn parse(rom: &[u8]) -> Header {
    let title = &rom[0x20..=0x34];
    let code = &rom[0x3b..=0x3e];
    let version = rom[0x3f];

    let ipl3_checksum = Crc::<u32>::new(&crc::CRC_32_CKSUM).checksum(&rom[0x0040..0x1000]);

    let cic_type = match ipl3_checksum {
        0x0013579c => CicType::Nus6101,
        0xd1f2d592 => CicType::Nus6102,
        0x27df61e2 => CicType::Nus6103,
        0x229f516c => CicType::Nus6105,
        0xa0dd69f7 => CicType::Nus6106,
        0x522fd8eb => CicType::MiniIPL3,
        _ => CicType::Unknown,
    };

    let code_without_region = String::from_utf8_lossy(&code[0..=2]);

    let save_type = *SAVE_TYPE_MAP
        .get(&code_without_region)
        .unwrap_or(&SaveType::Eeprom4K);

    debug!("Title: {}", String::from_utf8_lossy(title));
    debug!("Code: {}", String::from_utf8_lossy(code));
    debug!("Version: {}", version);
    debug!("CIC Type: {} (checksum: {})", cic_type, ipl3_checksum);
    debug!("Save Type: {}", save_type);

    Header {
        cic_type,
        save_type,
    }
}

impl Display for CicType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CicType::Unknown => "Unknown",
                CicType::Nus6101 => "NUS-6101",
                CicType::Nus6102 => "NUS-6102",
                CicType::Nus6103 => "NUS-6103",
                CicType::Nus6105 => "NUS-6105",
                CicType::Nus6106 => "NUS-6106",
                CicType::MiniIPL3 => "MiniIPL3",
            }
        )
    }
}
impl Display for SaveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SaveType::Eeprom4K => "EEPROM (4Kbit)",
                SaveType::Eeprom16K => "EEPROM (16Kbit)",
            }
        )
    }
}

const SAVE_TYPE_MAP: Map<&'static str, SaveType> = phf_map! {
    "NYS" => SaveType::Eeprom16K,
};
