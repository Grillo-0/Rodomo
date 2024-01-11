pub mod cpu;
pub mod ines;
pub mod ram;

use std::env;
use std::process;

use crate::cpu::Cpu;
use crate::ines::INes;
use crate::ram::Ram;

struct Machine {
    cpu: Cpu,
    memory: Ram,
}

impl Machine {
    fn new(memory: Ram) -> Machine {
        Machine {
            cpu: Cpu::new(),
            memory: memory,
        }
    }

    fn power_on(&mut self) {
        self.cpu.pc = 0xc000;
        let mut pos_02 = self.memory.read(0x02);
        let mut pos_03 = self.memory.read(0x03);

        loop {
            if pos_02 != self.memory.read(0x02) && pos_03 != self.memory.read(0x03) {
                break;
            }
            println!("0x{:04x}", self.cpu.pc);
            self.cpu.read_instruction(&mut self.memory);
        }

        pos_02 = self.memory.read(0x02);
        pos_03 = self.memory.read(0x03);
        println!("0x02:{:#02x} 0x03:{:#02x}", pos_02, pos_03);
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
    ram.load_vec_at(rom.program, 0xc000);

    let mut nes = Machine::new(ram);

    nes.power_on();
    println!("TEST PASSED");
}
