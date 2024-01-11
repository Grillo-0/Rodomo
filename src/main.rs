use std::env;
use std::process;

use crate::cpu::Cpu;
use crate::ram::Ram;

pub mod cpu;
pub mod ram;

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
        self.cpu.pc = 0x400;
        loop {
            if self.cpu.pc == 0x3469 {
                break;
            }

            self.cpu.read_instruction(&mut self.memory);
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

    let ram = Ram::from_raw_file(&file_name);
    let mut nes = Machine::new(ram);

    nes.power_on();
    println!("TEST PASSED");
}
