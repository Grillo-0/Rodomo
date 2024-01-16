use crate::Ram;

const NAMETABLE_MASK: u8 = 0b11;
const VRAM_MASK: u8 = 1 << 2;
const SPRITE_MASK: u8 = 1 << 3;
const BACKGROUND_MASK: u8 = 1 << 4;
const SPRITE_SIZE_MASK: u8 = 1 << 5;
const MASTER_SLAVE_MASK: u8 = 1 << 6;
const NMI_MASK: u8 = 1 << 7;

#[derive(Debug, Default)]
enum VramIncrement {
    #[default]
    Down,
    Across,
}

#[derive(Debug, Default)]
enum SpriteSize {
    #[default]
    Size8x8,
    Size8x16,
}

#[derive(Debug, Default)]
enum JobType {
    #[default]
    Read,
    Output,
}

#[derive(Debug, Default)]
pub struct Ppu {
    control: u8,
    mask: u8,
    status: u8,
    oam_addr: u8,
    oam_data: u8,
    scroll: u8,
    addr: u8,
    data: u8,
    oam_dma: u8,

    nametable_base: u16,
    vram_increment: VramIncrement,
    sprite_table_addr: u16,
    background_table_addr: u16,
    sprite_size: SpriteSize,
    job: JobType,

    memory: Ram,
}

impl Ppu {
    pub fn new(memory: Ram) -> Ppu {
        Ppu {
            memory,
            ..Default::default()
        }
    }

    pub fn read_instruction(&mut self, cpu_memory: &mut Ram) -> bool {
        self.control = cpu_memory.read(0x2000);
        self.mask = cpu_memory.read(0x2001);
        self.status = cpu_memory.read(0x2002);
        self.oam_addr = cpu_memory.read(0x2003);
        self.oam_data = cpu_memory.read(0x2004);
        self.scroll = cpu_memory.read(0x2005);
        self.addr = cpu_memory.read(0x2006);
        self.data = cpu_memory.read(0x2007);
        self.oam_dma = cpu_memory.read(0x4014);

        self.nametable_base = 0x2000 + 0x0400 * (self.control & NAMETABLE_MASK) as u16;
        self.vram_increment = if self.control & VRAM_MASK == 0 {
            VramIncrement::Across
        } else {
            VramIncrement::Down
        };
        self.sprite_table_addr = 0x0000 + 0x1000 * (self.control & SPRITE_MASK) as u16;
        self.background_table_addr = 0x0000 + 0x1000 * (self.control & BACKGROUND_MASK) as u16;
        self.sprite_size = if self.control & SPRITE_SIZE_MASK == 0 {
            SpriteSize::Size8x8
        } else {
            SpriteSize::Size8x16
        };
        self.job = if self.control & MASTER_SLAVE_MASK == 0 {
            JobType::Read
        } else {
            JobType::Output
        };

        return (self.control & NMI_MASK) != 0;
    }
}
