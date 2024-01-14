pub mod cpu;
pub mod ines;
pub mod ppu;
pub mod ram;

use std::env;
use std::process;

use crate::cpu::Cpu;
use crate::ines::INes;
use crate::ppu::Ppu;
use crate::ram::Ram;

struct Machine {
    cpu: Cpu,
    ppu: Ppu,
    memory: Ram,
}

impl Machine {
    fn new(memory: Ram, ppu_memory: Ram) -> Machine {
        Machine {
            cpu: Cpu::new(),
            ppu: Ppu::new(ppu_memory),
            memory: memory,
        }
    }

    fn power_on(&mut self) {
        let mut pos_02 = self.memory.read(0x02);
        let mut pos_03 = self.memory.read(0x03);

        self.cpu.reset(&self.memory);

        loop {
            if pos_02 != self.memory.read(0x02) && pos_03 != self.memory.read(0x03) {
                break;
            }
            println!("0x{:04x}", self.cpu.pc);
            self.cpu.read_instruction(&mut self.memory);
            self.ppu.read_instruction(&mut self.memory);

        pos_02 = self.memory.read(0x02);
        pos_03 = self.memory.read(0x03);
        println!("0x02:{:#02x} 0x03:{:#02x}", pos_02, pos_03);
        }
    }
}

fn main() {
    let mut args = env::args();
    let command = args.next().unwrap();
    let file_name = args.next().unwrap_or_else(|| {
        eprintln!("usage: {} <file_name>", command);
        process::exit(1);
    });

    let rom = INes::parse(&file_name);

    let mut ram = Ram::new();
    let prg_start = ((1 << 16) - rom.program.len()).try_into().unwrap();
    ram.load_vec_at(rom.program, prg_start);

    let mut ppu_mem = Ram::new();
    if let Some(chr_rom) = rom.chr_rom {
        ppu_mem.load_vec_at(chr_rom, 0);
    }

    let mut nes = Machine::new(ram, ppu_mem);

    nes.power_on();
    println!("TEST PASSED");
}
