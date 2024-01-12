use std::fs;

#[derive(Debug)]
pub struct INes {
    pub program: Vec<u8>,
}

impl INes {
    pub fn parse(path: &str) -> INes {
        let bytes = fs::read(path).expect("could not read file!");
        assert_eq!(String::from_utf8_lossy(&bytes[0..3]), "NES");

        let program_size = 16 * (1 << 10) * bytes[4] as usize;
        let program_rom_offset = 16;

        INes {
            program: bytes[(program_rom_offset)..(program_rom_offset + program_size)].to_vec(),
        }
    }
}
