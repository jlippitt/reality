use crc::Crc;
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

pub struct Header {
    pub cic_type: CicType,
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

    debug!("Title: {}", String::from_utf8_lossy(title));
    debug!("Code: {}", String::from_utf8_lossy(code));
    debug!("Version: {}", version);
    debug!("CIC Type: {} (checksum: {})", cic_type, ipl3_checksum);

    Header { cic_type }
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
