use glow::HasContext;

use crate::asc::MemoryMapped;
use crate::{gfx, Ram};

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

const CHARS_WIDTH: u32 = 16;
const CHARS_HEIGHT: u32 = 32;

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
    addr: u8,
    data: u8,
    oam_dma: u8,

    nametable_base: u16,
    vram_increment: VramIncrement,
    sprite_table_addr: u16,
    background_table_addr: u16,
    sprite_size: SpriteSize,
    job: JobType,
    nmi: bool,

    vblank: bool,

    memory: Ram,

    chars_texture: Option<glow::Texture>,
    char_program: Option<glow::Program>,
}

impl MemoryMapped for Ppu {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x2000 => {
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
                    0x1000
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
            0x2001 => self.mask = value,
            0x2002 => (),
            0x2003 => self.oam_addr = value,
            0x2004 => self.oam_data = value,
            0x2005 => self.scroll = value,
            0x2006 => self.addr = value,
            0x2007 => self.data = value,
            0x4014 => self.oam_dma = value,
            _ => panic!("Address {addr:#x} is not registered by the PPU"),
        }
    }

    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x2000 => self.control,
            0x2001 => self.mask,
            0x2002 => {
                let st = self.status;
                self.status &= !VBLANK_MASK;
                st
            }
            0x2003 => self.oam_addr,
            0x2004 => self.oam_data,
            0x2005 => self.scroll,
            0x2006 => self.addr,
            0x2007 => self.data,
            0x4014 => self.oam_dma,
            _ => panic!("Address {addr:#x} is not registered by the PPU"),
        }
    }
}

impl Ppu {
    pub fn new(memory: Ram) -> Ppu {
        Ppu {
            control: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            oam_data: 0,
            scroll: 0,
            addr: 0,
            data: 0,
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
        let chars_texture;
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

        chars = chars
            .into_iter()
            .flat_map(|i| {
                let pallete: [u32; 4] = [0x101010, 0xdda15e, 0x936639, 0xefefef];
                pallete[i as usize].to_le_bytes()[0..3].to_vec()
            })
            .collect();

        unsafe {
            chars_texture = gl.create_texture().unwrap();

            gl.bind_texture(glow::TEXTURE_2D, Some(chars_texture));

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as i32,
                CHAR_PIXEL_SIZE as i32,
                (CHAR_PIXEL_SIZE * NUM_CHARS) as i32,
                0,
                glow::BGR,
                glow::UNSIGNED_BYTE,
                Some(chars.as_slice()),
            );

            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.generate_mipmap(glow::TEXTURE_2D);

            let vert_shader = include_str!("../assets/char.vert");
            let frag_shader = include_str!("../assets/char.frag");
            char_program = gfx::create_program(gl, &vert_shader, &frag_shader);
        }

        (self.chars_texture, self.char_program) = (Some(chars_texture), Some(char_program));
    }

    unsafe fn draw_char(&self, gl: &glow::Context, index: usize, x: usize, y: usize) {
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
        let sampler = gl
            .get_uniform_location(self.char_program.unwrap(), "sampler")
            .unwrap();
        gl.uniform_1_i32(Some(&sampler), 0);

        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, &elements, glow::STATIC_DRAW);
        gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0);

        gl.delete_buffer(vbo);
        gl.delete_vertex_array(vao);
        gl.delete_buffer(ebo);
    }

    pub fn draw(&self, gl: &glow::Context) {
        unsafe {
            gl.clear(glow::COLOR_BUFFER_BIT);
            for y in 0..(CHARS_HEIGHT as usize) {
                for x in 0..(CHARS_WIDTH as usize) {
                    self.draw_char(gl, x + y * CHARS_WIDTH as usize, x, y);
                }
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
