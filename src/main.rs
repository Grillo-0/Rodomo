use std::env;
use std::process;

use crate::cpu::Cpu;
use crate::ram::Ram;

pub mod cpu;
pub mod ram;

fn main() {
    let mut args = env::args();
    let command = args.next().unwrap();
    let file_name = args.next().unwrap_or_else(|| {
        eprintln!("usage: {} <file_name>", command);
        process::exit(1);
    });
    let ram = Ram::from_raw_file(&file_name);
    let mut cpu = Cpu::create(ram);
    cpu.pc = 0x400;
    while cpu.pc != 0x3469 {
        cpu.read_instruction();
    }
    println!("TEST PASSED");
}
