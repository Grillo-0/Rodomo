extern crate sdl2;

pub mod cpu;
pub mod ines;
pub mod ppu;
pub mod ram;

use std::env;
use std::process;
use std::thread;
use std::time;

use sdl2::event::Event;
use sdl2::Sdl;

use crate::cpu::Cpu;
use crate::ines::INes;
use crate::ppu::Ppu;
use crate::ram::Ram;

struct Machine {
    cpu: Cpu,
    ppu: Ppu,
    memory: Ram,
    ctx: Sdl,
}

impl Machine {
    fn new(memory: Ram, ppu_memory: Ram, ctx: Sdl) -> Machine {
        Machine {
            cpu: Cpu::new(),
            ppu: Ppu::new(ppu_memory),
            memory: memory,
            ctx,
        }
    }

    fn power_on(&mut self) {
        let video = self.ctx.video().unwrap();

        let window = video
            .window("emulator", 800, 600)
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        let mut events = self.ctx.event_pump().unwrap();

        self.cpu.reset(&self.memory);

        loop {
            self.cpu.read_instruction(&mut self.memory);
            self.ppu.read_instruction(&mut self.memory);

            canvas.present();

            for e in events.poll_iter() {
                match e {
                    Event::Quit { .. } => process::exit(0),
                    _ => {}
                }
            }
            thread::sleep(time::Duration::from_secs_f64(1.0 / 60.0));
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

    let ctx = sdl2::init().unwrap();

    let mut nes = Machine::new(ram, ppu_mem, ctx);

    nes.power_on();
}
