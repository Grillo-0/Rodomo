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

        const FPS: f32 = 60.0;
        let frame_time = time::Duration::from_secs_f32(1.0 / FPS);

        const SCANLINES_PER_FRAME: u32 = 262;
        const PPU_CYCLES_PER_SCANLINE: u32 = 341;

        loop {
            let start = time::Instant::now();

            for scanline in 0..SCANLINES_PER_FRAME {
                let mut should_nmi = false;
                for tick in 0..PPU_CYCLES_PER_SCANLINE {
                    if tick % 3 == 0 {
                        self.cpu.read_instruction(&mut self.memory);
                    }

                    should_nmi = self.ppu.read_instruction(&mut self.memory)
                }

                if scanline == 241 && should_nmi {
                    self.cpu.nmi(&self.memory);
                }
            }
            canvas.present();

            for e in events.poll_iter() {
                match e {
                    Event::Quit { .. } => process::exit(0),
                    _ => {}
                }
            }

            let elapsed_time = start.elapsed();

            thread::sleep(frame_time.saturating_sub(elapsed_time));
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
