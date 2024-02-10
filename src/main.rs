pub mod asc;
pub mod cpu;
pub mod gfx;
pub mod ines;
pub mod ppu;
pub mod ram;

use std::cell::RefCell;
use std::env;
use std::process;
use std::rc::Rc;
use std::thread;
use std::time;

use glow::HasContext;
use sdl2::event::{Event, WindowEvent};

use crate::cpu::Cpu;
use crate::ines::INes;
use crate::ppu::Ppu;
use crate::ram::Ram;
use asc::Asc;

struct Machine {
    cpu: Cpu,
    ppu: Rc<RefCell<Ppu>>,
    asc: Asc,
}

impl Machine {
    fn new(memory: Ram, ppu_memory: Ram) -> Machine {
        let memory = Rc::new(RefCell::new(memory));
        let ppu = Rc::new(RefCell::new(Ppu::new(ppu_memory)));

        let mut asc = Asc::new();
        // TODO: Handle memory mirroring
        // Based on https://www.nesdev.org/wiki/CPU_memory_map
        asc.register_device_range(0x0000..=0x07ff, memory.clone()); // Internal RAM
        asc.register_device_range(0x2000..=0x2007, ppu.clone()); // PPU registers
        asc.register_device(0x4014, ppu.clone()); // OAM DMA
        asc.register_device_range(0x4020..=0xffff, memory); // Cartridge space

        Machine {
            cpu: Cpu::new(),
            ppu,
            asc,
        }
    }

    fn power_on(&mut self) {
        let (sdl, _video, window, gl, _gl_ctx) = gfx::setup();

        let mut events = sdl.event_pump().unwrap();

        self.cpu.reset(&mut self.asc);
        self.ppu.borrow_mut().precal_chars(&gl);
        self.ppu.borrow_mut().setup_pallet_tex(&gl);
        unsafe {
            gl.clear_color(0.1, 0.2, 0.3, 1.0);
        }

        const FPS: f32 = 60.0;
        let frame_time = time::Duration::from_secs_f32(1.0 / FPS);

        const SCANLINES_PER_FRAME: u32 = 262;
        const PPU_CYCLES_PER_SCANLINE: u32 = 341;

        loop {
            let start = time::Instant::now();

            for scanline in 0..SCANLINES_PER_FRAME {
                let cycles = self.cpu.cycles;
                for tick in 0..PPU_CYCLES_PER_SCANLINE {
                    if tick % 3 == 0 {
                        self.cpu.read_instruction(&mut self.asc);
                    }

                    if cycles.0.abs_diff(self.cpu.cycles.0) > (PPU_CYCLES_PER_SCANLINE / 3) as usize
                    {
                        break;
                    }
                }

                if scanline == 0 {
                    self.ppu.borrow_mut().reset_vblank();
                }

                if scanline == 241 {
                    self.ppu.borrow_mut().set_vblank();
                }

                if scanline == 241 && self.ppu.borrow().should_nmi() {
                    self.cpu.nmi(&mut self.asc);
                }
            }

            self.ppu.borrow_mut().draw(&gl);
            window.gl_swap_window();

            for e in events.poll_iter() {
                match e {
                    Event::Window {
                        timestamp: _,
                        window_id: _,
                        win_event,
                    } => match win_event {
                        WindowEvent::Resized(width, height) => {
                            unsafe {
                                gl.viewport(0, 0, width, height);
                            };
                        }
                        _ => {}
                    },
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

    let mut nes = Machine::new(ram, ppu_mem);

    nes.power_on();
}
