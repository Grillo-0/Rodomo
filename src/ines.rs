use std::fs;

#[derive(Debug)]
pub struct INes {
    pub program: Vec<u8>,
    pub chr_rom: Option<Vec<u8>>,
}

impl INes {
    pub fn parse(path: &str) -> INes {
        const TRAINER_MASK: u8 = 1 << 2;

        let bytes = fs::read(path).expect("could not read file!");
        assert_eq!(String::from_utf8_lossy(&bytes[0..3]), "NES");

        let flags_6 = bytes[6];

        let program_size = 16 * (1 << 10) * bytes[4] as usize;
        let program_rom_offset = 16 + 512 * (flags_6 & TRAINER_MASK) as usize;

        let chr_rom_size = 8 * (1 << 10) * bytes[5] as usize;
        let chr_rom_offset = program_rom_offset + program_size;

        INes {
            program: bytes[(program_rom_offset)..(program_rom_offset + program_size)].to_vec(),
            chr_rom: if program_size != 0 {
                Some(bytes[(chr_rom_offset)..(chr_rom_offset + chr_rom_size)].to_vec())
            } else {
                None
            },
        }
    }
}
