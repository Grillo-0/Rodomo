use std::cell::RefCell;
use std::rc::Rc;

use glow::HasContext;

use crate::asc::MemoryMapped;
use crate::{gfx, Asc, Ram};

const NAMETABLE_MASK: u8 = 0b11;
const VRAM_MASK: u8 = 1 << 2;
const SPRITE_MASK: u8 = 1 << 3;
const BACKGROUND_MASK: u8 = 1 << 4;
const SPRITE_SIZE_MASK: u8 = 1 << 5;
const MASTER_SLAVE_MASK: u8 = 1 << 6;
const NMI_MASK: u8 = 1 << 7;

const VBLANK_MASK: u8 = 1 << 7;

const NUM_CHARS: u32 = 512;
const CHAR_PIXEL_SIZE: u32 = 8;

const CHARS_WIDTH: u32 = 32;
const CHARS_HEIGHT: u32 = 30;

const ATRTABLE_SIZE: usize = 8;

#[rustfmt::skip]
const DEFAULT_SYSTEM_PALLETE: [u32; 64] = [
    0x626262, 0x002391, 0x1810A6, 0x440099, 0x660071, 0x6D002C, 0x680A00, 0x4D2400,
    0x2D3F00, 0x034B00, 0x005100, 0x004B1B, 0x003F62, 0x000000, 0x000000, 0x000000,
    0xABABAB, 0x1E55EA, 0x463BFF, 0x8220F6, 0xAD1CBD, 0xBA2061, 0xB33215, 0x8D5600,
    0x617900, 0x298B00, 0x079300, 0x008B4A, 0x0079A9, 0x000000, 0x000000, 0x000000,
    0xFFFFFF, 0x6CA6FF, 0x968BFF, 0xD46FFF, 0xFF6BFF, 0xFF6FB2, 0xFF8263, 0xE0A70C,
    0xB2CB00, 0x78DE00, 0x55E632, 0x3EDE9A, 0x4ECBFD, 0x4E4E4E, 0x000000, 0x000000,
    0xFFFFFF, 0xC4DBFF, 0xD5D0FF, 0xEEC5FF, 0xFFC6FF, 0xFFC5E0, 0xFFCDC0, 0xF3DC9D,
    0xE3ED96, 0xC9F299, 0xBBF5AD, 0xB2F2D7, 0xBBEDFF, 0xB8B8B8, 0x000000, 0x000000,
];

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

#[derive(Debug)]
pub struct Ppu {
    control: u8,
    mask: u8,
    status: u8,
    oam_addr: u8,
    oam_data: u8,
    scroll: u8,
    addr: u16,
    oam_dma: u8,

    nametable_base: u16,
    vram_increment: VramIncrement,
    sprite_table_addr: u16,
    background_table_addr: u16,
    sprite_size: SpriteSize,
    job: JobType,
    nmi: bool,

    vblank: bool,

    memory: Asc,

    chars_texture: Option<glow::Texture>,
    char_program: Option<glow::Program>,

    first_byte: bool,

    system_pallete_texture: Option<glow::Texture>,
}

impl MemoryMapped for Ppu {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0 => {
                self.control = value;
                self.nametable_base = 0x2000 + 0x0400 * (self.control & NAMETABLE_MASK) as u16;
                self.vram_increment = if self.control & VRAM_MASK == 0 {
                    VramIncrement::Across
                } else {
                    VramIncrement::Down
                };
                self.sprite_table_addr = if self.control & SPRITE_MASK == 0 {
                    0
                } else {
                    0x1000
                };
                self.background_table_addr = if self.control & BACKGROUND_MASK == 0 {
                    0
                } else {
                    0x100
                };
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
                self.nmi = self.control & NMI_MASK != 0;
            }
            0x1 => self.mask = value,
            0x2 => (),
            0x3 => self.oam_addr = value,
            0x4 => self.oam_data = value,
            0x5 => self.scroll = value,
            0x6 => {
                if !self.first_byte {
                    self.addr = (value as u16) << 8;
                } else {
                    self.addr |= value as u16;
                }
                self.first_byte = !self.first_byte;
            }
            0x7 => {
                self.memory.write(self.addr, value);
                self.addr += match self.vram_increment {
                    VramIncrement::Across => 1,
                    VramIncrement::Down => 32,
                }
            }
            0x4014 => self.oam_dma = value,
            _ => panic!("Address {addr:#x} is not registered by the PPU"),
        }
    }

    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0 => self.control,
            0x1 => self.mask,
            0x2 => {
                let st = self.status;
                self.status &= !VBLANK_MASK;
                self.first_byte = false;
                st
            }
            0x3 => self.oam_addr,
            0x4 => self.oam_data,
            0x5 => self.scroll,
            0x6 => self.addr as u8,
            0x7 => {
                let value = self.memory.read(self.addr);
                self.addr += match self.vram_increment {
                    VramIncrement::Across => 1,
                    VramIncrement::Down => 32,
                };
                value
            }
            0x4014 => self.oam_dma,
            _ => panic!("Address {addr:#x} is not registered by the PPU"),
        }
    }
}

impl Ppu {
    pub fn new(pattern_tables: Ram) -> Ppu {
        let mut memory = Asc::new();

        let pattern_tables = Rc::new(RefCell::new(pattern_tables));
        memory.register_device_range(0x0000..=0x1fff, pattern_tables, 0xffff);

        let nametables = Rc::new(RefCell::new(Ram::new()));
        memory.register_device_range(0x2000..=0x3eff, nametables, 0xfff);

        let pallettes = Rc::new(RefCell::new(Ram::new()));
        memory.register_device_range(0x3f00..=0x3fff, pallettes.clone(), 0x1f);
        memory.register_device(0x3f10, pallettes.clone(), 0xf);
        memory.register_device(0x3f14, pallettes.clone(), 0xf);
        memory.register_device(0x3f18, pallettes.clone(), 0xf);
        memory.register_device(0x3f1c, pallettes, 0xf);

        Ppu {
            control: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            oam_data: 0,
            scroll: 0,
            addr: 0,
            oam_dma: 0,

            nametable_base: 0,
            vram_increment: VramIncrement::default(),
            sprite_table_addr: 0,
            background_table_addr: 0,
            sprite_size: SpriteSize::default(),
            job: JobType::default(),
            nmi: false,

            vblank: false,

            memory,
            chars_texture: None,
            char_program: None,

            first_byte: false,

            system_pallete_texture: None,
        }
    }

    fn decode_char(char: [u8; 16]) -> Vec<u8> {
        let mut decoded = vec![];
        let (plane0, plane1) = char.split_at(8);
        for (p0, p1) in plane0.iter().zip(plane1) {
            for i in (0..8).rev() {
                let bit0 = (p0 >> i) & 1;
                let bit1 = (p1 >> i) & 1;
                let pixel = (bit1 << 1) | bit0;
                decoded.push(pixel);
            }
        }
        return decoded;
    }

    pub fn precal_chars(&mut self, gl: &glow::Context) {
        let char_program;

        let mut chars: Vec<u8> = vec![];

        for i in 0..(NUM_CHARS as u16) {
            let mut char = vec![];
            for addr in 0..16 {
                char.push(self.memory.read(addr + 16 * i));
            }
            let mut char = Ppu::decode_char(char.try_into().unwrap());
            chars.append(&mut char);
        }

        let chars_texture = gfx::create_tex(
            gl,
            glow::TEXTURE_2D,
            glow::R8 as i32,
            CHAR_PIXEL_SIZE as i32,
            (CHAR_PIXEL_SIZE * NUM_CHARS) as i32,
            glow::RED,
            chars.as_slice(),
        );

        unsafe {
            let vert_shader = include_str!("../assets/char.vert");
            let frag_shader = include_str!("../assets/char.frag");
            char_program = gfx::create_program(gl, &vert_shader, &frag_shader);
        }

        (self.chars_texture, self.char_program) = (Some(chars_texture), Some(char_program));
    }

    pub fn setup_pallet_tex(&mut self, gl: &glow::Context) {
        self.system_pallete_texture = Some(gfx::create_tex(
            gl,
            glow::TEXTURE_1D,
            glow::RGB as i32,
            64,
            0,
            glow::BGR,
            &DEFAULT_SYSTEM_PALLETE
                .iter()
                .flat_map(|v: &u32| {
                    let v: [u8; 4] = v.to_le_bytes();
                    v[0..3].to_vec()
                })
                .collect::<Vec<u8>>(),
        ));
    }

    unsafe fn draw_char(
        &self,
        gl: &glow::Context,
        index: usize,
        x: usize,
        y: usize,
        palletes_tex: glow::Texture,
        atrtable_tex: glow::Texture,
    ) {
        let mut quad: Vec<f32> = vec![
            1.0f32, 1.0f32, 1.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 1.0f32,
        ];

        let mut quad_tex = quad.clone();

        let x: f32 = x as f32 / CHARS_WIDTH as f32;
        let y: f32 = y as f32 / CHARS_HEIGHT as f32;

        quad = quad
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let mut p = *p;
                if i % 2 == 0 {
                    p /= CHARS_WIDTH as f32;
                    p + x
                } else {
                    p /= CHARS_HEIGHT as f32;
                    p + y
                }
            })
            .collect::<Vec<f32>>()
            .try_into()
            .unwrap();

        quad_tex = quad_tex
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let mut p = *p;
                if i % 2 == 0 {
                    p
                } else {
                    p += index as f32;
                    p / NUM_CHARS as f32
                }
            })
            .collect::<Vec<f32>>()
            .try_into()
            .unwrap();

        quad.append(&mut quad_tex);

        let verts = core::slice::from_raw_parts(
            quad.as_ptr() as *const u8,
            quad.len() * core::mem::size_of::<f32>(),
        );

        let vbo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &verts, glow::STATIC_DRAW);

        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));

        let pos_loc = gl
            .get_attrib_location(self.char_program.unwrap(), "pos")
            .unwrap();
        gl.enable_vertex_attrib_array(pos_loc);
        gl.vertex_attrib_pointer_f32(
            pos_loc,
            2,
            glow::FLOAT,
            false,
            2 * core::mem::size_of::<f32>() as i32,
            0,
        );

        let pos_uv = gl
            .get_attrib_location(self.char_program.unwrap(), "uv_in")
            .unwrap();
        gl.enable_vertex_attrib_array(pos_uv);
        gl.vertex_attrib_pointer_f32(
            pos_uv,
            2,
            glow::FLOAT,
            false,
            2 * core::mem::size_of::<f32>() as i32,
            8 * core::mem::size_of::<f32>() as i32,
        );

        let elements: [u32; 6] = [0, 1, 3, 1, 2, 3];
        let elements = core::slice::from_raw_parts(
            elements.as_ptr() as *const u8,
            elements.len() * core::mem::size_of::<u32>(),
        );
        let ebo = gl.create_buffer().unwrap();
        gl.use_program(self.char_program);

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, self.chars_texture);
        let mut sampler = gl
            .get_uniform_location(self.char_program.unwrap(), "chars_sampler")
            .unwrap();
        gl.uniform_1_i32(Some(&sampler), 0);

        gl.active_texture(glow::TEXTURE1);
        gl.bind_texture(glow::TEXTURE_1D, self.system_pallete_texture);
        sampler = gl
            .get_uniform_location(self.char_program.unwrap(), "syspallete_sampler")
            .unwrap();
        gl.uniform_1_i32(Some(&sampler), 1);

        gl.active_texture(glow::TEXTURE2);
        gl.bind_texture(glow::TEXTURE_1D, Some(palletes_tex));
        sampler = gl
            .get_uniform_location(self.char_program.unwrap(), "palletes_sampler")
            .unwrap();
        gl.uniform_1_i32(Some(&sampler), 2);

        gl.active_texture(glow::TEXTURE3);
        gl.bind_texture(glow::TEXTURE_2D, Some(atrtable_tex));
        let sampler = gl
            .get_uniform_location(self.char_program.unwrap(), "atrtable_sampler")
            .unwrap();
        gl.uniform_1_i32(Some(&sampler), 3);

        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, &elements, glow::STATIC_DRAW);
        gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0);

        gl.delete_buffer(vbo);
        gl.delete_vertex_array(vao);
        gl.delete_buffer(ebo);
    }

    pub fn draw(&mut self, gl: &glow::Context) {
        let mut pallets = vec![];
        for i in 0x3f00..=0x3f0f {
            pallets.push(self.memory.read(i));
        }

        let pallets_tex = gfx::create_tex(
            gl,
            glow::TEXTURE_1D,
            glow::R8 as i32,
            16,
            0,
            glow::RED,
            pallets.as_slice(),
        );

        let atrtable_offset = CHARS_WIDTH * CHARS_HEIGHT;
        let atrtable_addr = self.nametable_base + atrtable_offset as u16;

        let mut attribute_table = vec![];

        for i in 0..(ATRTABLE_SIZE * ATRTABLE_SIZE) as u16 {
            attribute_table.push(self.memory.read(atrtable_addr + i));
        }

        let mut buf = vec![0; (ATRTABLE_SIZE * ATRTABLE_SIZE * 4) as usize];

        for (i, v) in attribute_table.into_iter().enumerate() {
            let top_left = (v >> (0 * 2)) & 0x3;
            let top_right = (v >> (1 * 2)) & 0x3;
            let bottom_left = (v >> (2 * 2)) & 0x3;
            let bottom_right = (v >> (3 * 2)) & 0x3;

            let mut x = i % ATRTABLE_SIZE;
            let mut y = i / ATRTABLE_SIZE;
            x *= 2;
            y *= 2;
            let mut pos = x + y * (CHARS_WIDTH / 2) as usize;
            buf[pos] = top_left;

            x += 1;
            pos = x + y * (CHARS_WIDTH / 2) as usize;
            buf[pos] = top_right;

            y += 1;
            pos = x + y * (CHARS_WIDTH / 2) as usize;
            buf[pos] = bottom_right;

            x -= 1;
            pos = x + y * (CHARS_WIDTH / 2) as usize;
            buf[pos] = bottom_left;
        }

        buf.truncate((CHARS_WIDTH * CHARS_HEIGHT / 4) as usize);
        let attribute_table = buf;

        let atrtable_tex = gfx::create_tex(
            gl,
            glow::TEXTURE_2D,
            glow::R8 as i32,
            (CHARS_WIDTH / 2) as i32,
            (CHARS_HEIGHT / 2) as i32,
            glow::RED,
            attribute_table.as_slice(),
        );

        unsafe {
            gl.clear(glow::COLOR_BUFFER_BIT);

            for i in 0..CHARS_WIDTH * CHARS_HEIGHT {
                let char = self.memory.read(i as u16 + self.nametable_base) as usize
                    + self.background_table_addr as usize;
                self.draw_char(
                    gl,
                    char,
                    (i % CHARS_WIDTH) as usize,
                    (i / CHARS_WIDTH) as usize,
                    pallets_tex,
                    atrtable_tex,
                );
            }
        }
    }

    pub fn reset_vblank(&mut self) {
        self.vblank = false;
        self.status &= !VBLANK_MASK;
    }

    pub fn set_vblank(&mut self) {
        self.vblank = true;
        self.status |= VBLANK_MASK;
    }

    pub fn should_nmi(&self) -> bool {
        self.vblank && self.nmi
    }
}
