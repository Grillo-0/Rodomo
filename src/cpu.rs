use std::fmt;

use crate::ram::Ram;

const NEGATIVE_MASK: u8 = 1 << 7;

#[derive(Debug, Default)]
pub struct Cpu {
    sp: u8,      // Stack Pointer
    pub pc: u16, // Program Counter

    a: u8, // Accumulator
    x: u8, // Index Register X
    y: u8, // Index Register Y

    carry_flag: bool,
    zero_flag: bool,
    interrupt_flag: bool,
    decimal_flag: bool,
    break_cmd_flag: bool,
    reserved_flag: bool,
    overflow_flag: bool,
    negative_flag: bool,
}

macro_rules! instr {
    ($instruction:ident-imp) => {
        |cpu: &mut Cpu, ram: &mut Ram| {
            cpu.$instruction(ram);
        }
    };
    ($instruction:ident-$addr_mode:ident) => {
        |cpu: &mut Cpu, ram: &mut Ram| {
            let addr = cpu.$addr_mode(ram);
            cpu.$instruction(addr, ram);
        }
    };
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "stack pointer: {:#02X}\n", self.sp).ok();
        write!(f, "program counter: {:#02X}\n", self.pc).ok();
        write!(f, "A: {:#02X}\n", self.a).ok();
        write!(f, "X: {:#02X}\n", self.x).ok();
        write!(f, "Y: {:#02X}\n", self.y).ok();
        write!(f, "Carry: {}\n", self.carry_flag).ok();
        write!(f, "Zero: {}\n", self.zero_flag).ok();
        write!(f, "Interrupt: {}\n", self.interrupt_flag).ok();
        write!(f, "Decimal Mode: {}\n", self.decimal_flag).ok();
        write!(f, "Break: {}\n", self.break_cmd_flag).ok();
        write!(f, "Overflow: {}\n", self.overflow_flag).ok();
        write!(f, "Negative: {}", self.negative_flag)
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            sp: 0xff,
            ..Default::default()
        }
    }

    pub fn read_instruction(&mut self, ram: &mut Ram) {
        let opcode = ram.read(self.pc.into());
        self.get_instruction(opcode)(self, ram);
    }

    fn get_instruction(&mut self, opcode: u8) -> fn(&mut Cpu, &mut Ram) {
        match opcode {
            0xEA => instr!(nop - imp),
            0x1A => instr!(nop - imp),
            0x3A => instr!(nop - imp),
            0x5A => instr!(nop - imp),
            0x7A => instr!(nop - imp),
            0xDA => instr!(nop - imp),
            0xFA => instr!(nop - imp),
            0x80 => instr!(nop_addr - imm),
            0x82 => instr!(nop_addr - imm),
            0x89 => instr!(nop_addr - imm),
            0xC2 => instr!(nop_addr - imm),
            0xE2 => instr!(nop_addr - imm),
            0x04 => instr!(nop_addr - zp),
            0x44 => instr!(nop_addr - zp),
            0x64 => instr!(nop_addr - zp),
            0x14 => instr!(nop_addr - zpx),
            0x34 => instr!(nop_addr - zpx),
            0x54 => instr!(nop_addr - zpx),
            0x74 => instr!(nop_addr - zpx),
            0xD4 => instr!(nop_addr - zpx),
            0xF4 => instr!(nop_addr - zpx),
            0x0C => instr!(nop_addr - abs),
            0x1C => instr!(nop_addr - abx),
            0x3C => instr!(nop_addr - abx),
            0x5C => instr!(nop_addr - abx),
            0x7C => instr!(nop_addr - abx),
            0xDC => instr!(nop_addr - abx),
            0xFC => instr!(nop_addr - abx),

            0xA9 => instr!(lda - imm),
            0xA5 => instr!(lda - zp),
            0xB5 => instr!(lda - zpx),
            0xAD => instr!(lda - abs),
            0xBD => instr!(lda - abx),
            0xB9 => instr!(lda - aby),
            0xA1 => instr!(lda - inx),
            0xB1 => instr!(lda - iny),

            0xA2 => instr!(ldx - imm),
            0xA6 => instr!(ldx - zp),
            0xB6 => instr!(ldx - zpy),
            0xAE => instr!(ldx - abs),
            0xBE => instr!(ldx - aby),

            0xA0 => instr!(ldy - imm),
            0xA4 => instr!(ldy - zp),
            0xB4 => instr!(ldy - zpx),
            0xAC => instr!(ldy - abs),
            0xBC => instr!(ldy - abx),

            0xA7 => instr!(lax - zp),
            0xB7 => instr!(lax - zpy),
            0xAF => instr!(lax - abs),
            0xBF => instr!(lax - aby),
            0xA3 => instr!(lax - inx),
            0xB3 => instr!(lax - iny),

            0x85 => instr!(sta - zp),
            0x95 => instr!(sta - zpx),
            0x8D => instr!(sta - abs),
            0x9D => instr!(sta - abx),
            0x99 => instr!(sta - aby),
            0x81 => instr!(sta - inx),
            0x91 => instr!(sta - iny),

            0x86 => instr!(stx - zp),
            0x96 => instr!(stx - zpy),
            0x8E => instr!(stx - abs),

            0x84 => instr!(sty - zp),
            0x94 => instr!(sty - zpx),
            0x8C => instr!(sty - abs),

            0x87 => instr!(sax - zp),
            0x97 => instr!(sax - zpy),
            0x8F => instr!(sax - abs),
            0x83 => instr!(sax - inx),

            0xAA => instr!(tax - imp),

            0xA8 => instr!(tay - imp),

            0x8A => instr!(txa - imp),

            0x98 => instr!(tya - imp),

            0xBA => instr!(tsx - imp),

            0x9A => instr!(txs - imp),

            0x48 => instr!(pha - imp),

            0x08 => instr!(php - imp),

            0x68 => instr!(pla - imp),

            0x28 => instr!(plp - imp),

            0x29 => instr!(and - imm),
            0x25 => instr!(and - zp),
            0x35 => instr!(and - zpx),
            0x2D => instr!(and - abs),
            0x3D => instr!(and - abx),
            0x39 => instr!(and - aby),
            0x21 => instr!(and - inx),
            0x31 => instr!(and - iny),

            0x49 => instr!(eor - imm),
            0x45 => instr!(eor - zp),
            0x55 => instr!(eor - zpx),
            0x4D => instr!(eor - abs),
            0x5D => instr!(eor - abx),
            0x59 => instr!(eor - aby),
            0x41 => instr!(eor - inx),
            0x51 => instr!(eor - iny),

            0x09 => instr!(ora - imm),
            0x05 => instr!(ora - zp),
            0x15 => instr!(ora - zpx),
            0x0D => instr!(ora - abs),
            0x1D => instr!(ora - abx),
            0x19 => instr!(ora - aby),
            0x01 => instr!(ora - inx),
            0x11 => instr!(ora - iny),

            0x24 => instr!(bit - zp),
            0x2C => instr!(bit - abs),

            0x4C => instr!(jmp - abs),
            0x6C => instr!(jmp - ind),

            0x20 => instr!(jsr - abs),

            0x60 => instr!(rts - imp),

            0xD0 => instr!(bne - zp),
            0xF0 => instr!(beq - zp),
            0x10 => instr!(bpl - zp),
            0x90 => instr!(bcc - zp),
            0xB0 => instr!(bcs - zp),
            0x30 => instr!(bmi - zp),
            0x50 => instr!(bvc - zp),
            0x70 => instr!(bvs - zp),

            0xCA => instr!(dex - imp),
            0x88 => instr!(dey - imp),

            0xE8 => instr!(incx - imp),
            0xC8 => instr!(incy - imp),

            0x0A => instr!(asl - imp),
            0x06 => instr!(asl_addr - zp),
            0x16 => instr!(asl_addr - zpx),
            0x0E => instr!(asl_addr - abs),
            0x1E => instr!(asl_addr - abx),

            0x07 => instr!(slo - zp),
            0x17 => instr!(slo - zpx),
            0x0F => instr!(slo - abs),
            0x1F => instr!(slo - abx),
            0x1B => instr!(slo - aby),
            0x03 => instr!(slo - inx),
            0x13 => instr!(slo - iny),

            0x4A => instr!(lsr - imp),
            0x46 => instr!(lsr_addr - zp),
            0x56 => instr!(lsr_addr - zpx),
            0x4E => instr!(lsr_addr - abs),
            0x5E => instr!(lsr_addr - abx),

            0x47 => instr!(sre - zp),
            0x57 => instr!(sre - zpx),
            0x4F => instr!(sre - abs),
            0x5F => instr!(sre - abx),
            0x5B => instr!(sre - aby),
            0x43 => instr!(sre - inx),
            0x53 => instr!(sre - iny),

            0x2A => instr!(rol - imp),
            0x26 => instr!(rol_addr - zp),
            0x36 => instr!(rol_addr - zpx),
            0x2E => instr!(rol_addr - abs),
            0x3E => instr!(rol_addr - abx),

            0x27 => instr!(rla - zp),
            0x37 => instr!(rla - zpx),
            0x2F => instr!(rla - abs),
            0x3F => instr!(rla - abx),
            0x3B => instr!(rla - aby),
            0x23 => instr!(rla - inx),
            0x33 => instr!(rla - iny),

            0x6A => instr!(ror - imp),
            0x66 => instr!(ror_addr - zp),
            0x76 => instr!(ror_addr - zpx),
            0x6E => instr!(ror_addr - abs),
            0x7E => instr!(ror_addr - abx),

            0x67 => instr!(rra - zp),
            0x77 => instr!(rra - zpx),
            0x6F => instr!(rra - abs),
            0x7F => instr!(rra - abx),
            0x7B => instr!(rra - aby),
            0x63 => instr!(rra - inx),
            0x73 => instr!(rra - iny),

            0x18 => instr!(clc - imp),
            0x38 => instr!(sec - imp),

            0xD8 => instr!(cld - imp),
            0xF8 => instr!(sed - imp),

            0x58 => instr!(cli - imp),
            0x78 => instr!(sei - imp),

            0xB8 => instr!(clv - imp),

            0xC9 => instr!(cmp - imm),
            0xC5 => instr!(cmp - zp),
            0xD5 => instr!(cmp - zpx),
            0xCD => instr!(cmp - abs),
            0xDD => instr!(cmp - abx),
            0xD9 => instr!(cmp - aby),
            0xC1 => instr!(cmp - inx),
            0xD1 => instr!(cmp - iny),

            0xE0 => instr!(cpx - imm),
            0xE4 => instr!(cpx - zp),
            0xEC => instr!(cpx - abs),

            0xC0 => instr!(cpy - imm),
            0xC4 => instr!(cpy - zp),
            0xCC => instr!(cpy - abs),

            0x69 => instr!(adc - imm),
            0x65 => instr!(adc - zp),
            0x75 => instr!(adc - zpx),
            0x6D => instr!(adc - abs),
            0x7D => instr!(adc - abx),
            0x79 => instr!(adc - aby),
            0x61 => instr!(adc - inx),
            0x71 => instr!(adc - iny),

            0xE9 => instr!(sbc - imm),
            0xEB => instr!(sbc - imm),
            0xE5 => instr!(sbc - zp),
            0xF5 => instr!(sbc - zpx),
            0xED => instr!(sbc - abs),
            0xFD => instr!(sbc - abx),
            0xF9 => instr!(sbc - aby),
            0xE1 => instr!(sbc - inx),
            0xF1 => instr!(sbc - iny),

            0x00 => instr!(brk - imp),

            0x40 => instr!(rti - imp),

            0xE6 => instr!(inc - zp),
            0xF6 => instr!(inc - zpx),
            0xEE => instr!(inc - abs),
            0xFE => instr!(inc - abx),

            0xE7 => instr!(isc - zp),
            0xF7 => instr!(isc - zpx),
            0xEF => instr!(isc - abs),
            0xFF => instr!(isc - abx),
            0xFB => instr!(isc - aby),
            0xE3 => instr!(isc - inx),
            0xF3 => instr!(isc - iny),

            0xC6 => instr!(dec - zp),
            0xD6 => instr!(dec - zpx),
            0xCE => instr!(dec - abs),
            0xDE => instr!(dec - abx),

            0xC7 => instr!(dcp - zp),
            0xD7 => instr!(dcp - zpx),
            0xCF => instr!(dcp - abs),
            0xDF => instr!(dcp - abx),
            0xDB => instr!(dcp - aby),
            0xC3 => instr!(dcp - inx),
            0xD3 => instr!(dcp - iny),

            _ => unimplemented!("{:#04X} opcode not implemented yet!\n", opcode),
        }
    }

    fn imm(&mut self, _: &mut Ram) -> u16 {
        self.pc += 1;
        self.pc
    }

    fn zp(&mut self, ram: &mut Ram) -> u16 {
        self.pc += 1;
        ram.read(self.pc) as u16
    }

    fn zpx(&mut self, ram: &mut Ram) -> u16 {
        self.pc += 1;
        (ram.read(self.pc) as u16).wrapping_add(self.x as u16) & 0xff
    }

    fn zpy(&mut self, ram: &mut Ram) -> u16 {
        self.pc += 1;
        (ram.read(self.pc) as u16).wrapping_add(self.y as u16) & 0xff
    }

    fn abs(&mut self, ram: &mut Ram) -> u16 {
        self.pc += 1;
        let addr = ram.read(self.pc);
        self.pc += 1;
        (ram.read(self.pc) as u16) << 8 | addr as u16
    }

    fn abx(&mut self, ram: &mut Ram) -> u16 {
        self.pc += 1;
        let mut addr = ram.read(self.pc) as u16;
        self.pc += 1;
        addr |= (ram.read(self.pc) as u16) << 8;
        addr.wrapping_add(self.x as u16)
    }

    fn aby(&mut self, ram: &mut Ram) -> u16 {
        self.pc += 1;
        let mut addr = ram.read(self.pc) as u16;
        self.pc += 1;
        addr |= (ram.read(self.pc) as u16) << 8;
        addr.wrapping_add(self.y as u16)
    }

    fn inx(&mut self, ram: &mut Ram) -> u16 {
        self.pc += 1;
        let mut addr: u16 = ram.read(self.pc) as u16;
        addr = (addr.wrapping_add(self.x as u16) & 0xff) as u16;
        (ram.read(addr + 1) as u16) << 8 | ram.read(addr.into()) as u16
    }

    fn iny(&mut self, ram: &mut Ram) -> u16 {
        self.pc += 1;
        let addr: u16 = ram.read(self.pc) as u16;
        let addr = (ram.read(addr.wrapping_add(1)) as u16) << 8 | ram.read(addr) as u16;
        addr.wrapping_add(self.y as u16)
    }

    fn ind(&mut self, ram: &mut Ram) -> u16 {
        self.pc += 1;
        let addr = ram.read(self.pc);
        self.pc += 1;
        let addr = (ram.read(self.pc) as u16) << 8 | addr as u16;
        (ram.read(addr + 1) as u16) << 8 | ram.read(addr.into()) as u16
    }

    fn push(&mut self, value: u8, ram: &mut Ram) {
        ram.write(0x0100 | self.sp as u16, value);

        self.sp = self.sp.wrapping_sub(1);
    }

    fn pop(&mut self, ram: &mut Ram) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        ram.read(0x0100 | self.sp as u16)
    }

    fn push_long(&mut self, value: u16, ram: &mut Ram) {
        self.push(((value >> 8) & 0xff).try_into().unwrap(), ram);
        self.push((value & 0xff).try_into().unwrap(), ram);
    }

    fn pop_long(&mut self, ram: &mut Ram) -> u16 {
        let mut addr = self.pop(ram) as u16;
        addr |= (self.pop(ram) as u16) << 8;
        return addr;
    }

    fn nop(&mut self, _: &mut Ram) {
        self.pc += 1;
    }

    fn nop_addr(&mut self, _: u16, _: &mut Ram) {
        self.pc += 1;
    }

    fn lda(&mut self, addr: u16, ram: &mut Ram) {
        self.a = ram.read(addr);

        self.zero_flag = self.a == 0;

        self.negative_flag = self.a & NEGATIVE_MASK != 0;

        self.pc += 1;
    }

    fn ldx(&mut self, addr: u16, ram: &mut Ram) {
        self.x = ram.read(addr);

        self.zero_flag = self.x == 0;
        self.negative_flag = self.x & NEGATIVE_MASK != 0;

        self.pc += 1;
    }

    fn ldy(&mut self, addr: u16, ram: &mut Ram) {
        self.y = ram.read(addr);

        self.zero_flag = self.y == 0;
        self.negative_flag = self.y & NEGATIVE_MASK != 0;

        self.pc += 1;
    }

    fn lax(&mut self, addr: u16, ram: &mut Ram) {
        let value = ram.read(addr);
        self.x = value;
        self.a = value;

        self.zero_flag = self.a == 0;
        self.negative_flag = self.a & NEGATIVE_MASK != 0;

        self.pc += 1;
    }

    fn sta(&mut self, addr: u16, ram: &mut Ram) {
        ram.write(addr, self.a);

        self.pc += 1;
    }

    fn stx(&mut self, addr: u16, ram: &mut Ram) {
        ram.write(addr, self.x);

        self.pc += 1;
    }

    fn sty(&mut self, addr: u16, ram: &mut Ram) {
        ram.write(addr, self.y);

        self.pc += 1;
    }

    fn sax(&mut self, addr: u16, ram: &mut Ram) {
        ram.write(addr, self.a & self.x);

        self.pc += 1;
    }

    fn tax(&mut self, _: &mut Ram) {
        self.x = self.a;

        self.zero_flag = self.x == 0;
        self.negative_flag = self.x & NEGATIVE_MASK != 0;

        self.pc += 1;
    }

    fn tay(&mut self, _: &mut Ram) {
        self.y = self.a;

        self.zero_flag = self.y == 0;
        self.negative_flag = self.y & NEGATIVE_MASK != 0;

        self.pc += 1;
    }

    fn txa(&mut self, _: &mut Ram) {
        self.a = self.x;

        self.zero_flag = self.a == 0;
        self.negative_flag = self.a & NEGATIVE_MASK != 0;

        self.pc += 1;
    }

    fn tya(&mut self, _: &mut Ram) {
        self.a = self.y;

        self.zero_flag = self.a == 0;
        self.negative_flag = self.a & NEGATIVE_MASK != 0;

        self.pc += 1;
    }

    fn tsx(&mut self, _: &mut Ram) {
        self.x = self.sp;

        self.zero_flag = self.x == 0;
        self.negative_flag = self.x & NEGATIVE_MASK != 0;

        self.pc += 1;
    }

    fn txs(&mut self, _: &mut Ram) {
        self.sp = self.x;

        self.pc += 1;
    }

    fn pha(&mut self, ram: &mut Ram) {
        self.push(self.a, ram);

        self.pc += 1;
    }

    fn status_to_word(&self) -> u8 {
        let mut status = 0;

        status |= (self.carry_flag as u8) << 0;
        status |= (self.zero_flag as u8) << 1;
        status |= (self.interrupt_flag as u8) << 2;
        status |= (self.decimal_flag as u8) << 3;
        status |= (self.break_cmd_flag as u8) << 4;
        status |= (self.reserved_flag as u8) << 5;
        status |= (self.overflow_flag as u8) << 6;
        status |= (self.negative_flag as u8) << 7;

        return status;
    }

    fn php(&mut self, ram: &mut Ram) {
        // TODO: Find if this is realy correct
        self.reserved_flag = true;
        self.break_cmd_flag = true;

        self.push(self.status_to_word(), ram);

        self.pc += 1;
    }

    fn pla(&mut self, ram: &mut Ram) {
        self.a = self.pop(ram);

        self.zero_flag = self.a == 0;
        self.negative_flag = self.a & NEGATIVE_MASK != 0;

        self.pc += 1;
    }

    fn word_to_status(&mut self, word: u8) {
        self.carry_flag = ((word >> 0) & 1) != 0;
        self.zero_flag = ((word >> 1) & 1) != 0;
        self.interrupt_flag = ((word >> 2) & 1) != 0;
        self.decimal_flag = ((word >> 3) & 1) != 0;
        self.break_cmd_flag = ((word >> 4) & 1) != 0;
        self.reserved_flag = ((word >> 5) & 1) != 0;
        self.overflow_flag = ((word >> 6) & 1) != 0;
        self.negative_flag = ((word >> 7) & 1) != 0;
    }

    fn plp(&mut self, ram: &mut Ram) {
        let status = self.pop(ram);
        self.word_to_status(status);

        self.pc += 1;
    }

    fn and(&mut self, addr: u16, ram: &mut Ram) {
        self.a &= ram.read(addr);

        self.zero_flag = self.a == 0;
        self.negative_flag = self.a & NEGATIVE_MASK != 0;

        self.pc += 1;
    }

    fn eor(&mut self, addr: u16, ram: &mut Ram) {
        self.a ^= ram.read(addr);

        self.zero_flag = self.a == 0;
        self.negative_flag = self.a & NEGATIVE_MASK != 0;

        self.pc += 1;
    }

    fn ora(&mut self, addr: u16, ram: &mut Ram) {
        self.a |= ram.read(addr);

        self.zero_flag = self.a == 0;
        self.negative_flag = self.a & NEGATIVE_MASK != 0;

        self.pc += 1;
    }

    fn bit(&mut self, addr: u16, ram: &mut Ram) {
        let value = ram.read(addr);

        self.zero_flag = self.a & value == 0;
        self.overflow_flag = value & 1 << 6 != 0;
        self.negative_flag = value & NEGATIVE_MASK != 0;

        self.pc += 1;
    }

    fn jmp(&mut self, addr: u16, _: &mut Ram) {
        self.pc = addr;
    }

    fn jsr(&mut self, addr: u16, ram: &mut Ram) {
        self.push_long(self.pc, ram);
        self.pc = addr;
    }

    fn rts(&mut self, ram: &mut Ram) {
        let addr = self.pop_long(ram);
        self.pc = addr.wrapping_add(1);
    }

    fn bne(&mut self, addr: u16, _: &mut Ram) {
        if !self.zero_flag {
            self.pc = self.pc.wrapping_add_signed((addr as i8) as i16);
        }

        self.pc += 1;
    }

    fn beq(&mut self, addr: u16, _: &mut Ram) {
        if self.zero_flag {
            self.pc = self.pc.wrapping_add_signed((addr as i8) as i16);
        }

        self.pc += 1;
    }

    fn bpl(&mut self, addr: u16, _: &mut Ram) {
        if !self.negative_flag {
            self.pc = self.pc.wrapping_add_signed((addr as i8) as i16);
        }

        self.pc += 1;
    }

    fn bcc(&mut self, addr: u16, _: &mut Ram) {
        if !self.carry_flag {
            self.pc = self.pc.wrapping_add_signed((addr as i8) as i16);
        }

        self.pc += 1;
    }

    fn bcs(&mut self, addr: u16, _: &mut Ram) {
        if self.carry_flag {
            self.pc = self.pc.wrapping_add_signed((addr as i8) as i16);
        }

        self.pc += 1;
    }

    fn bmi(&mut self, addr: u16, _: &mut Ram) {
        if self.negative_flag {
            self.pc = self.pc.wrapping_add_signed((addr as i8) as i16);
        }

        self.pc += 1;
    }

    fn bvc(&mut self, addr: u16, _: &mut Ram) {
        if !self.overflow_flag {
            self.pc = self.pc.wrapping_add_signed((addr as i8) as i16);
        }

        self.pc += 1;
    }

    fn bvs(&mut self, addr: u16, _: &mut Ram) {
        if self.overflow_flag {
            self.pc = self.pc.wrapping_add_signed((addr as i8) as i16);
        }

        self.pc += 1;
    }

    fn dex(&mut self, _: &mut Ram) {
        self.x = self.x.wrapping_sub(1);

        self.zero_flag = self.x == 0;
        self.negative_flag = (self.x >> 7) & 1 == 1;

        self.pc += 1;
    }

    fn dey(&mut self, _: &mut Ram) {
        self.y = self.y.wrapping_sub(1);

        self.zero_flag = self.y == 0;
        self.negative_flag = (self.y >> 7) & 1 == 1;

        self.pc += 1;
    }

    fn incx(&mut self, _: &mut Ram) {
        self.x = self.x.wrapping_add(1);

        self.zero_flag = self.x == 0;
        self.negative_flag = (self.x >> 7) & 1 == 1;

        self.pc += 1;
    }

    fn incy(&mut self, _: &mut Ram) {
        self.y = self.y.wrapping_add(1);

        self.zero_flag = self.y == 0;
        self.negative_flag = (self.y >> 7) & 1 == 1;

        self.pc += 1;
    }

    fn shift_left(&mut self, mut value: u8) -> u8 {
        self.carry_flag = value & NEGATIVE_MASK != 0;
        value <<= 1;
        self.negative_flag = value & NEGATIVE_MASK != 0;
        self.zero_flag = value == 0;

        self.pc += 1;
        return value;
    }

    fn asl(&mut self, _: &mut Ram) {
        self.a = self.shift_left(self.a);
    }

    fn asl_addr(&mut self, addr: u16, ram: &mut Ram) {
        let mut value = ram.read(addr);
        value = self.shift_left(value);
        ram.write(addr, value);
    }

    fn slo(&mut self, addr: u16, ram: &mut Ram) {
        self.asl_addr(addr, ram);
        self.pc = self.pc.wrapping_sub(1);
        self.ora(addr, ram);
    }

    fn shift_right(&mut self, mut value: u8) -> u8 {
        self.carry_flag = value & 1 != 0;
        value >>= 1;
        self.negative_flag = value & NEGATIVE_MASK != 0;
        self.zero_flag = value == 0;

        self.pc += 1;
        return value;
    }

    fn lsr(&mut self, _: &mut Ram) {
        self.a = self.shift_right(self.a);
    }

    fn lsr_addr(&mut self, addr: u16, ram: &mut Ram) {
        let mut value = ram.read(addr);
        value = self.shift_right(value);
        ram.write(addr, value);
    }

    fn sre(&mut self, addr: u16, ram: &mut Ram) {
        self.lsr_addr(addr, ram);
        self.pc = self.pc.wrapping_sub(1);
        self.eor(addr, ram);
    }

    fn rotate_left(&mut self, mut value: u8) -> u8 {
        let old = value;
        value <<= 1;
        value |= self.carry_flag as u8;

        self.carry_flag = old & NEGATIVE_MASK != 0;
        self.negative_flag = value & NEGATIVE_MASK != 0;
        self.zero_flag = value == 0;

        self.pc += 1;
        return value;
    }

    fn rol(&mut self, _: &mut Ram) {
        self.a = self.rotate_left(self.a);
    }

    fn rol_addr(&mut self, addr: u16, ram: &mut Ram) {
        let mut value = ram.read(addr);
        value = self.rotate_left(value);
        ram.write(addr, value);
    }

    fn rla(&mut self, addr: u16, ram: &mut Ram) {
        self.rol_addr(addr, ram);
        self.pc = self.pc.wrapping_sub(1);
        self.and(addr, ram);
    }

    fn rotate_right(&mut self, mut value: u8) -> u8 {
        let old = value;
        value >>= 1;
        value |= (self.carry_flag as u8) << 7;

        self.carry_flag = old & 1 != 0;
        self.negative_flag = value & NEGATIVE_MASK != 0;
        self.zero_flag = value == 0;

        self.pc += 1;
        return value;
    }

    fn ror(&mut self, _: &mut Ram) {
        self.a = self.rotate_right(self.a);
    }

    fn ror_addr(&mut self, addr: u16, ram: &mut Ram) {
        let mut value = ram.read(addr);
        value = self.rotate_right(value);
        ram.write(addr, value);
    }

    fn rra(&mut self, addr: u16, ram: &mut Ram) {
        self.ror_addr(addr, ram);
        self.pc = self.pc.wrapping_sub(1);
        self.adc(addr, ram);
    }

    fn clc(&mut self, _: &mut Ram) {
        self.carry_flag = false;

        self.pc += 1;
    }

    fn sec(&mut self, _: &mut Ram) {
        self.carry_flag = true;

        self.pc += 1;
    }

    fn cld(&mut self, _: &mut Ram) {
        self.decimal_flag = false;

        self.pc += 1;
    }

    fn sed(&mut self, _: &mut Ram) {
        self.decimal_flag = true;

        self.pc += 1;
    }

    fn cli(&mut self, _: &mut Ram) {
        self.interrupt_flag = false;

        self.pc += 1;
    }

    fn sei(&mut self, _: &mut Ram) {
        self.interrupt_flag = true;

        self.pc += 1;
    }

    fn clv(&mut self, _: &mut Ram) {
        self.overflow_flag = false;

        self.pc += 1;
    }

    fn cmp(&mut self, addr: u16, ram: &mut Ram) {
        let value = ram.read(addr);

        let res = self.a.wrapping_sub(value);

        self.carry_flag = self.a >= value;
        self.zero_flag = self.a == value;
        self.negative_flag = res & NEGATIVE_MASK != 0;

        self.pc += 1;
    }

    fn cpx(&mut self, addr: u16, ram: &mut Ram) {
        let value = ram.read(addr);

        let res = self.x.wrapping_sub(value);

        self.carry_flag = self.x >= value;
        self.zero_flag = self.x == value;
        self.negative_flag = res & NEGATIVE_MASK != 0;

        self.pc += 1;
    }

    fn cpy(&mut self, addr: u16, ram: &mut Ram) {
        let value = ram.read(addr);

        let res = self.y.wrapping_sub(value);

        self.carry_flag = self.y >= value;
        self.zero_flag = self.y == value;
        self.negative_flag = res & NEGATIVE_MASK != 0;

        self.pc += 1;
    }

    fn add_with_carry(&mut self, value: u8) {
        let t1 = self.a.wrapping_add(value);
        let c = t1 < self.a;
        let t2 = t1.wrapping_add(self.carry_flag as u8);
        self.carry_flag = c | (t2 < t1);
        self.overflow_flag = (!(self.a ^ value) & (self.a ^ t2)) >> 7 != 0;
        self.a = t2;
        self.zero_flag = self.a == 0;
        self.negative_flag = self.a & NEGATIVE_MASK != 0;
    }

    fn adc(&mut self, addr: u16, ram: &mut Ram) {
        let value = ram.read(addr);

        self.add_with_carry(value);

        self.pc += 1;
    }

    fn sbc(&mut self, addr: u16, ram: &mut Ram) {
        let value = ram.read(addr);

        self.add_with_carry(!value);

        self.pc += 1;
    }

    fn brk(&mut self, ram: &mut Ram) {
        self.push_long(self.pc + 2, ram);
        self.break_cmd_flag = true;
        self.reserved_flag = true;
        self.push(self.status_to_word(), ram);

        let mut addr: u16 = ram.read(0xfffe) as u16;
        addr |= (ram.read(0xffff) as u16) << 8;

        self.pc = addr;
        self.interrupt_flag = true;
    }

    fn rti(&mut self, ram: &mut Ram) {
        let word = self.pop(ram);
        self.word_to_status(word);
        self.pc = self.pop_long(ram);
    }

    fn inc(&mut self, addr: u16, ram: &mut Ram) {
        let mut value = ram.read(addr);
        value = value.wrapping_add(1);
        ram.write(addr, value);

        self.zero_flag = value == 0;
        self.negative_flag = value & NEGATIVE_MASK != 0;

        self.pc = self.pc.wrapping_add(1);
    }

    fn isc(&mut self, addr: u16, ram: &mut Ram) {
        self.inc(addr, ram);
        self.pc = self.pc.wrapping_sub(1);
        self.sbc(addr, ram);
    }

    fn dec(&mut self, addr: u16, ram: &mut Ram) {
        let mut value = ram.read(addr);
        value = value.wrapping_sub(1);
        ram.write(addr, value);

        self.zero_flag = value == 0;
        self.negative_flag = value & NEGATIVE_MASK != 0;

        self.pc = self.pc.wrapping_add(1);
    }

    fn dcp(&mut self, addr: u16, ram: &mut Ram) {
        self.dec(addr, ram);
        self.pc = self.pc.wrapping_sub(1);
        self.cmp(addr, ram);
    }
}
