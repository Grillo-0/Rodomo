use crate::Ram;

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

        return true;
    }
}
