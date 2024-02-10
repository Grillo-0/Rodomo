use std::fmt;
use std::num::Wrapping;

use crate::asc::{Asc, MemoryMapped};

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

    pub cycles: Wrapping<usize>,
}

#[derive(Debug, Clone)]
enum AddressingMode {
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndexedIndirect,
    IndirectIndexed,
}

#[derive(Debug)]
enum InstructionKind {
    Nop,
    Lda,
    Ldx,
    Ldy,
    Lax,
    Sta,
    Stx,
    Sty,
    Sax,
    Tax,
    Tay,
    Txa,
    Tya,
    Tsx,
    Txs,
    Pha,
    Php,
    Pla,
    Plp,
    And,
    Eor,
    Ora,
    Bit,
    Jmp,
    Jsr,
    Rts,
    Bne,
    Beq,
    Bpl,
    Bcc,
    Bcs,
    Bmi,
    Bvc,
    Bvs,
    Dex,
    Dey,
    Inc,
    Incx,
    Incy,
    Asl,
    AslAddr,
    Slo,
    Lsr,
    LsrAddr,
    Sre,
    Rol,
    RolAddr,
    Rla,
    Ror,
    RorAddr,
    Rra,
    Clc,
    Sec,
    Cld,
    Sed,
    Cli,
    Sei,
    Clv,
    Cmp,
    Cpx,
    Cpy,
    Adc,
    Sbc,
    Brk,
    Rti,
    Isc,
    Dec,
    Dcp,
}

#[derive(Debug)]
struct Instruction {
    kind: InstructionKind,
    addr_mode: AddressingMode,
    cycles: u32,
}

trait InstructionTrait {
    fn instr(cpu: &mut Cpu, addr: u16, mem: &mut Asc);

    fn run_with(addr_mode: AddressingMode, cpu: &mut Cpu, mem: &mut Asc) -> Instruction;
}

macro_rules! impl_instr {
    ($instruction:ident, $instruction_logic:expr, $cycles: expr) => {
        struct $instruction;
        impl InstructionTrait for $instruction {
            fn instr(cpu: &mut Cpu, addr: u16, mem: &mut Asc) {
                $instruction_logic(cpu, addr, mem);
            }

            fn run_with(addr_mode: AddressingMode, cpu: &mut Cpu, mem: &mut Asc) -> Instruction {
                use AddressingMode::*;
                let addr = match addr_mode {
                    Implicit | Accumulator => 0,
                    Immediate => cpu.imm(mem),
                    ZeroPage | Relative => cpu.zp(mem),
                    ZeroPageX => cpu.zpx(mem),
                    ZeroPageY => cpu.zpy(mem),
                    Absolute => cpu.abs(mem),
                    AbsoluteX => cpu.abx(mem),
                    AbsoluteY => cpu.aby(mem),
                    Indirect => cpu.ind(mem),
                    IndirectIndexed => cpu.inx(mem),
                    IndexedIndirect => cpu.iny(mem),
                };

                Self::instr(cpu, addr, mem);
                let cycles = $cycles(addr_mode.clone());
                Instruction {
                    kind: InstructionKind::$instruction,
                    addr_mode,
                    cycles,
                }
            }
        }
    };
}

impl_instr!(
    Nop,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 2,
            Immediate => 2,
            ZeroPage => 3,
            ZeroPageX => 4,
            Absolute => 4,
            AbsoluteX => 4,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Lda,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        cpu.a = mem.read(addr);

        cpu.zero_flag = cpu.a == 0;

        cpu.negative_flag = cpu.a & NEGATIVE_MASK != 0;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Immediate => 2,
            ZeroPage => 3,
            ZeroPageX => 4,
            Absolute => 4,
            AbsoluteX => 4,
            AbsoluteY => 4,
            IndirectIndexed => 6,
            IndexedIndirect => 5,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Ldx,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        cpu.x = mem.read(addr);

        cpu.zero_flag = cpu.x == 0;
        cpu.negative_flag = cpu.x & NEGATIVE_MASK != 0;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Immediate => 2,
            ZeroPage => 3,
            ZeroPageY => 4,
            Absolute => 4,
            AbsoluteY => 4,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Ldy,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        cpu.y = mem.read(addr);

        cpu.zero_flag = cpu.y == 0;
        cpu.negative_flag = cpu.y & NEGATIVE_MASK != 0;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Immediate => 2,
            ZeroPage => 3,
            ZeroPageX => 4,
            Absolute => 4,
            AbsoluteX => 4,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Lax,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        let value = mem.read(addr);
        cpu.x = value;
        cpu.a = value;

        cpu.zero_flag = cpu.a == 0;
        cpu.negative_flag = cpu.a & NEGATIVE_MASK != 0;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            ZeroPage => 3,
            ZeroPageY => 4,
            Absolute => 4,
            AbsoluteY => 4,
            IndexedIndirect => 6,
            IndirectIndexed => 5,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Sta,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        mem.write(addr, cpu.a);

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            ZeroPage => 3,
            ZeroPageX => 4,
            Absolute => 4,
            AbsoluteX => 5,
            AbsoluteY => 5,
            IndexedIndirect => 6,
            IndirectIndexed => 6,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Stx,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        mem.write(addr, cpu.x);

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            ZeroPage => 3,
            ZeroPageY => 4,
            Absolute => 4,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Sty,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        mem.write(addr, cpu.y);

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            ZeroPage => 3,
            ZeroPageX => 4,
            Absolute => 4,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Sax,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        mem.write(addr, cpu.a & cpu.x);

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            ZeroPage => 3,
            ZeroPageY => 4,
            Absolute => 4,
            IndirectIndexed => 6,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Tax,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.x = cpu.a;

        cpu.zero_flag = cpu.x == 0;
        cpu.negative_flag = cpu.x & NEGATIVE_MASK != 0;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Tay,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.y = cpu.a;

        cpu.zero_flag = cpu.y == 0;
        cpu.negative_flag = cpu.y & NEGATIVE_MASK != 0;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Txa,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.a = cpu.x;

        cpu.zero_flag = cpu.a == 0;
        cpu.negative_flag = cpu.a & NEGATIVE_MASK != 0;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Tya,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.a = cpu.y;

        cpu.zero_flag = cpu.a == 0;
        cpu.negative_flag = cpu.a & NEGATIVE_MASK != 0;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Tsx,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.x = cpu.sp;

        cpu.zero_flag = cpu.x == 0;
        cpu.negative_flag = cpu.x & NEGATIVE_MASK != 0;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Txs,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.sp = cpu.x;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Pha,
    |cpu: &mut Cpu, _addr: u16, mem: &mut Asc| {
        cpu.push(cpu.a, mem);

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 3,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Php,
    |cpu: &mut Cpu, _addr: u16, mem: &mut Asc| {
        // TODO: Find if this is realy correct
        cpu.reserved_flag = true;
        cpu.break_cmd_flag = true;

        cpu.push(cpu.status_to_word(), mem);

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 3,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Pla,
    |cpu: &mut Cpu, _addr: u16, mem: &mut Asc| {
        cpu.a = cpu.pop(mem);

        cpu.zero_flag = cpu.a == 0;
        cpu.negative_flag = cpu.a & NEGATIVE_MASK != 0;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 4,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Plp,
    |cpu: &mut Cpu, _addr: u16, mem: &mut Asc| {
        let status = cpu.pop(mem);
        cpu.word_to_status(status);

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 4,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    And,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        cpu.a &= mem.read(addr);

        cpu.zero_flag = cpu.a == 0;
        cpu.negative_flag = cpu.a & NEGATIVE_MASK != 0;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Immediate => 2,
            ZeroPage => 3,
            ZeroPageX => 4,
            ZeroPageY => 4,
            Absolute => 4,
            AbsoluteX => 4,
            AbsoluteY => 4,
            IndirectIndexed => 6,
            IndexedIndirect => 5,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Eor,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        cpu.a ^= mem.read(addr);

        cpu.zero_flag = cpu.a == 0;
        cpu.negative_flag = cpu.a & NEGATIVE_MASK != 0;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Immediate => 2,
            ZeroPage => 3,
            ZeroPageX => 4,
            ZeroPageY => 4,
            Absolute => 4,
            AbsoluteX => 4,
            AbsoluteY => 4,
            IndexedIndirect => 6,
            IndirectIndexed => 5,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Ora,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        cpu.a |= mem.read(addr);

        cpu.zero_flag = cpu.a == 0;
        cpu.negative_flag = cpu.a & NEGATIVE_MASK != 0;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Immediate => 2,
            ZeroPage => 3,
            ZeroPageX => 4,
            ZeroPageY => 4,
            Absolute => 4,
            AbsoluteX => 4,
            AbsoluteY => 4,
            IndexedIndirect => 6,
            IndirectIndexed => 5,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Bit,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        let value = mem.read(addr);

        cpu.zero_flag = cpu.a & value == 0;
        cpu.overflow_flag = value & 1 << 6 != 0;
        cpu.negative_flag = value & NEGATIVE_MASK != 0;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            ZeroPage => 3,
            Absolute => 4,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Jmp,
    |cpu: &mut Cpu, addr: u16, _mem: &mut Asc| {
        cpu.pc = addr;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Absolute => 3,
            Indirect => 5,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Jsr,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        cpu.push_long(cpu.pc, mem);
        cpu.pc = addr;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Absolute => 6,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Rts,
    |cpu: &mut Cpu, _addr: u16, mem: &mut Asc| {
        let addr = cpu.pop_long(mem);
        cpu.pc = addr.wrapping_add(1);
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 6,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Bne,
    |cpu: &mut Cpu, addr: u16, _mem: &mut Asc| {
        if !cpu.zero_flag {
            cpu.pc = cpu.pc.wrapping_add_signed((addr as i8) as i16);
        }

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Relative => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Beq,
    |cpu: &mut Cpu, addr: u16, _mem: &mut Asc| {
        if cpu.zero_flag {
            cpu.pc = cpu.pc.wrapping_add_signed((addr as i8) as i16);
        }

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Relative => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Bpl,
    |cpu: &mut Cpu, addr: u16, _mem: &mut Asc| {
        if !cpu.negative_flag {
            cpu.pc = cpu.pc.wrapping_add_signed((addr as i8) as i16);
        }

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Relative => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Bcc,
    |cpu: &mut Cpu, addr: u16, _mem: &mut Asc| {
        if !cpu.carry_flag {
            cpu.pc = cpu.pc.wrapping_add_signed((addr as i8) as i16);
        }

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Relative => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Bcs,
    |cpu: &mut Cpu, addr: u16, _mem: &mut Asc| {
        if cpu.carry_flag {
            cpu.pc = cpu.pc.wrapping_add_signed((addr as i8) as i16);
        }

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Relative => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Bmi,
    |cpu: &mut Cpu, addr: u16, _mem: &mut Asc| {
        if cpu.negative_flag {
            cpu.pc = cpu.pc.wrapping_add_signed((addr as i8) as i16);
        }

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Relative => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Bvc,
    |cpu: &mut Cpu, addr: u16, _mem: &mut Asc| {
        if !cpu.overflow_flag {
            cpu.pc = cpu.pc.wrapping_add_signed((addr as i8) as i16);
        }

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Relative => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Bvs,
    |cpu: &mut Cpu, addr: u16, _mem: &mut Asc| {
        if cpu.overflow_flag {
            cpu.pc = cpu.pc.wrapping_add_signed((addr as i8) as i16);
        }

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Relative => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Dex,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.x = cpu.x.wrapping_sub(1);

        cpu.zero_flag = cpu.x == 0;
        cpu.negative_flag = (cpu.x >> 7) & 1 == 1;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Dey,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.y = cpu.y.wrapping_sub(1);

        cpu.zero_flag = cpu.y == 0;
        cpu.negative_flag = (cpu.y >> 7) & 1 == 1;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Inc,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        let mut value = mem.read(addr);
        value = value.wrapping_add(1);
        mem.write(addr, value);

        cpu.zero_flag = value == 0;
        cpu.negative_flag = value & NEGATIVE_MASK != 0;

        cpu.pc = cpu.pc.wrapping_add(1);
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            ZeroPage => 5,
            ZeroPageX => 6,
            Absolute => 6,
            AbsoluteX => 7,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Incx,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.x = cpu.x.wrapping_add(1);

        cpu.zero_flag = cpu.x == 0;
        cpu.negative_flag = (cpu.x >> 7) & 1 == 1;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Incy,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.y = cpu.y.wrapping_add(1);

        cpu.zero_flag = cpu.y == 0;
        cpu.negative_flag = (cpu.y >> 7) & 1 == 1;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Asl,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.a = cpu.shift_left(cpu.a);
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Accumulator => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    AslAddr,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        let mut value = mem.read(addr);
        value = cpu.shift_left(value);
        mem.write(addr, value);
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            ZeroPage => 5,
            ZeroPageX => 6,
            Absolute => 6,
            AbsoluteX => 7,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Slo,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        AslAddr::instr(cpu, addr, mem);
        cpu.pc = cpu.pc.wrapping_sub(1);
        Ora::instr(cpu, addr, mem);
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            ZeroPage => 5,
            ZeroPageX => 6,
            Absolute => 6,
            AbsoluteX => 7,
            AbsoluteY => 7,
            IndexedIndirect => 8,
            IndirectIndexed => 8,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Lsr,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.a = cpu.shift_right(cpu.a);
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Accumulator => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    LsrAddr,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        let mut value = mem.read(addr);
        value = cpu.shift_right(value);
        mem.write(addr, value);
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            ZeroPage => 5,
            ZeroPageX => 6,
            Absolute => 6,
            AbsoluteX => 7,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Sre,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        LsrAddr::instr(cpu, addr, mem);
        cpu.pc = cpu.pc.wrapping_sub(1);
        Eor::instr(cpu, addr, mem);
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            ZeroPage => 5,
            ZeroPageX => 6,
            Absolute => 6,
            AbsoluteX => 7,
            AbsoluteY => 7,
            IndexedIndirect => 8,
            IndirectIndexed => 8,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Rol,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.a = cpu.rotate_left(cpu.a);
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Accumulator => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    RolAddr,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        let mut value = mem.read(addr);
        value = cpu.rotate_left(value);
        mem.write(addr, value);
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            ZeroPage => 5,
            ZeroPageX => 6,
            Absolute => 6,
            AbsoluteX => 7,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Rla,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        RolAddr::instr(cpu, addr, mem);
        cpu.pc = cpu.pc.wrapping_sub(1);
        And::instr(cpu, addr, mem);
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            ZeroPage => 5,
            ZeroPageX => 6,
            Absolute => 6,
            AbsoluteX => 7,
            AbsoluteY => 7,
            IndexedIndirect => 8,
            IndirectIndexed => 8,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Ror,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.a = cpu.rotate_right(cpu.a);
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Accumulator => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    RorAddr,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        let mut value = mem.read(addr);
        value = cpu.rotate_right(value);
        mem.write(addr, value);
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            ZeroPage => 5,
            ZeroPageX => 6,
            Absolute => 6,
            AbsoluteX => 7,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Rra,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        RorAddr::instr(cpu, addr, mem);
        cpu.pc = cpu.pc.wrapping_sub(1);
        Adc::instr(cpu, addr, mem);
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            ZeroPage => 5,
            ZeroPageX => 6,
            Absolute => 6,
            AbsoluteX => 7,
            AbsoluteY => 7,
            IndexedIndirect => 8,
            IndirectIndexed => 8,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Clc,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.carry_flag = false;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Sec,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.carry_flag = true;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Cld,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.decimal_flag = false;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Sed,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.decimal_flag = true;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Cli,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.interrupt_flag = false;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Sei,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.interrupt_flag = true;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Clv,
    |cpu: &mut Cpu, _addr: u16, _mem: &mut Asc| {
        cpu.overflow_flag = false;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 2,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Cmp,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        let value = mem.read(addr);

        let res = cpu.a.wrapping_sub(value);

        cpu.carry_flag = cpu.a >= value;
        cpu.zero_flag = cpu.a == value;
        cpu.negative_flag = res & NEGATIVE_MASK != 0;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Immediate => 2,
            ZeroPage => 3,
            ZeroPageX => 4,
            Absolute => 4,
            AbsoluteX => 4,
            AbsoluteY => 4,
            IndexedIndirect => 6,
            IndirectIndexed => 5,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Cpx,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        let value = mem.read(addr);

        let res = cpu.x.wrapping_sub(value);

        cpu.carry_flag = cpu.x >= value;
        cpu.zero_flag = cpu.x == value;
        cpu.negative_flag = res & NEGATIVE_MASK != 0;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Immediate => 2,
            ZeroPage => 3,
            Absolute => 4,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Cpy,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        let value = mem.read(addr);

        let res = cpu.y.wrapping_sub(value);

        cpu.carry_flag = cpu.y >= value;
        cpu.zero_flag = cpu.y == value;
        cpu.negative_flag = res & NEGATIVE_MASK != 0;

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Immediate => 2,
            ZeroPage => 3,
            Absolute => 4,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Adc,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        let value = mem.read(addr);

        cpu.add_with_carry(value);

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Immediate => 2,
            ZeroPage => 3,
            ZeroPageX => 4,
            Absolute => 4,
            AbsoluteX => 4,
            AbsoluteY => 4,
            IndexedIndirect => 6,
            IndirectIndexed => 5,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Sbc,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        let value = mem.read(addr);

        cpu.add_with_carry(!value);

        cpu.pc += 1;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Immediate => 2,
            ZeroPage => 3,
            ZeroPageX => 4,
            Absolute => 4,
            AbsoluteX => 4,
            AbsoluteY => 4,
            IndexedIndirect => 6,
            IndirectIndexed => 5,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Brk,
    |cpu: &mut Cpu, _addr: u16, mem: &mut Asc| {
        cpu.push_long(cpu.pc + 2, mem);
        cpu.break_cmd_flag = true;
        cpu.reserved_flag = true;
        cpu.push(cpu.status_to_word(), mem);

        let mut addr: u16 = mem.read(0xfffe) as u16;
        addr |= (mem.read(0xffff) as u16) << 8;

        cpu.pc = addr;
        cpu.interrupt_flag = true;
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 7,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Rti,
    |cpu: &mut Cpu, _addr: u16, mem: &mut Asc| {
        let word = cpu.pop(mem);
        cpu.word_to_status(word);
        cpu.pc = cpu.pop_long(mem);
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            Implicit => 6,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Isc,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        Inc::instr(cpu, addr, mem);
        cpu.pc = cpu.pc.wrapping_sub(1);
        Sbc::instr(cpu, addr, mem);
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            ZeroPage => 5,
            ZeroPageX => 6,
            Absolute => 6,
            AbsoluteX => 7,
            AbsoluteY => 7,
            IndexedIndirect => 8,
            IndirectIndexed => 8,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Dec,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        let mut value = mem.read(addr);
        value = value.wrapping_sub(1);
        mem.write(addr, value);

        cpu.zero_flag = value == 0;
        cpu.negative_flag = value & NEGATIVE_MASK != 0;

        cpu.pc = cpu.pc.wrapping_add(1);
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            ZeroPage => 5,
            ZeroPageX => 6,
            Absolute => 6,
            AbsoluteX => 7,
            _ => unimplemented!(),
        }
    }
);

impl_instr!(
    Dcp,
    |cpu: &mut Cpu, addr: u16, mem: &mut Asc| {
        Dec::instr(cpu, addr, mem);
        cpu.pc = cpu.pc.wrapping_sub(1);
        Cmp::instr(cpu, addr, mem);
    },
    |addr_mode: AddressingMode| {
        use AddressingMode::*;
        match addr_mode {
            ZeroPage => 5,
            ZeroPageX => 6,
            Absolute => 6,
            AbsoluteX => 7,
            AbsoluteY => 7,
            IndexedIndirect => 8,
            IndirectIndexed => 8,
            _ => unimplemented!(),
        }
    }
);

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

    pub fn reset(&mut self, ram: &mut Asc) {
        let mut reset_addr = ram.read(0xfffc) as u16;
        reset_addr |= (ram.read(0xfffd) as u16) << 8;
        self.pc = reset_addr;
    }

    pub fn nmi(&mut self, ram: &mut Asc) {
        let mut nmi_addr = ram.read(0xfffa) as u16;
        nmi_addr |= (ram.read(0xfffb) as u16) << 8;

        self.push_long(self.pc, ram);
        self.push(self.status_to_word(), ram);

        self.pc = nmi_addr;
    }

    pub fn read_instruction(&mut self, ram: &mut Asc) {
        let opcode = ram.read(self.pc.into());
        self.run_instruction(opcode, ram);
    }

    fn run_instruction(&mut self, opcode: u8, mem: &mut Asc) -> Instruction {
        let instr = match opcode {
            0xEA => Nop::run_with(AddressingMode::Implicit, self, mem),
            0x1A => Nop::run_with(AddressingMode::Implicit, self, mem),
            0x3A => Nop::run_with(AddressingMode::Implicit, self, mem),
            0x5A => Nop::run_with(AddressingMode::Implicit, self, mem),
            0x7A => Nop::run_with(AddressingMode::Implicit, self, mem),
            0xDA => Nop::run_with(AddressingMode::Implicit, self, mem),
            0xFA => Nop::run_with(AddressingMode::Implicit, self, mem),
            0x80 => Nop::run_with(AddressingMode::Immediate, self, mem),
            0x82 => Nop::run_with(AddressingMode::Immediate, self, mem),
            0x89 => Nop::run_with(AddressingMode::Immediate, self, mem),
            0xC2 => Nop::run_with(AddressingMode::Immediate, self, mem),
            0xE2 => Nop::run_with(AddressingMode::Immediate, self, mem),
            0x04 => Nop::run_with(AddressingMode::ZeroPage, self, mem),
            0x44 => Nop::run_with(AddressingMode::ZeroPage, self, mem),
            0x64 => Nop::run_with(AddressingMode::ZeroPage, self, mem),
            0x14 => Nop::run_with(AddressingMode::ZeroPageX, self, mem),
            0x34 => Nop::run_with(AddressingMode::ZeroPageX, self, mem),
            0x54 => Nop::run_with(AddressingMode::ZeroPageX, self, mem),
            0x74 => Nop::run_with(AddressingMode::ZeroPageX, self, mem),
            0xD4 => Nop::run_with(AddressingMode::ZeroPageX, self, mem),
            0xF4 => Nop::run_with(AddressingMode::ZeroPageX, self, mem),
            0x0C => Nop::run_with(AddressingMode::Absolute, self, mem),
            0x1C => Nop::run_with(AddressingMode::AbsoluteX, self, mem),
            0x3C => Nop::run_with(AddressingMode::AbsoluteX, self, mem),
            0x5C => Nop::run_with(AddressingMode::AbsoluteX, self, mem),
            0x7C => Nop::run_with(AddressingMode::AbsoluteX, self, mem),
            0xDC => Nop::run_with(AddressingMode::AbsoluteX, self, mem),
            0xFC => Nop::run_with(AddressingMode::AbsoluteX, self, mem),

            0xA9 => Lda::run_with(AddressingMode::Immediate, self, mem),
            0xA5 => Lda::run_with(AddressingMode::ZeroPage, self, mem),
            0xB5 => Lda::run_with(AddressingMode::ZeroPageX, self, mem),
            0xAD => Lda::run_with(AddressingMode::Absolute, self, mem),
            0xBD => Lda::run_with(AddressingMode::AbsoluteX, self, mem),
            0xB9 => Lda::run_with(AddressingMode::AbsoluteY, self, mem),
            0xA1 => Lda::run_with(AddressingMode::IndirectIndexed, self, mem),
            0xB1 => Lda::run_with(AddressingMode::IndexedIndirect, self, mem),

            0xA2 => Ldx::run_with(AddressingMode::Immediate, self, mem),
            0xA6 => Ldx::run_with(AddressingMode::ZeroPage, self, mem),
            0xB6 => Ldx::run_with(AddressingMode::ZeroPageY, self, mem),
            0xAE => Ldx::run_with(AddressingMode::Absolute, self, mem),
            0xBE => Ldx::run_with(AddressingMode::AbsoluteY, self, mem),

            0xA0 => Ldy::run_with(AddressingMode::Immediate, self, mem),
            0xA4 => Ldy::run_with(AddressingMode::ZeroPage, self, mem),
            0xB4 => Ldy::run_with(AddressingMode::ZeroPageX, self, mem),
            0xAC => Ldy::run_with(AddressingMode::Absolute, self, mem),
            0xBC => Ldy::run_with(AddressingMode::AbsoluteX, self, mem),

            0xA7 => Lax::run_with(AddressingMode::ZeroPage, self, mem),
            0xB7 => Lax::run_with(AddressingMode::ZeroPageY, self, mem),
            0xAF => Lax::run_with(AddressingMode::Absolute, self, mem),
            0xBF => Lax::run_with(AddressingMode::AbsoluteY, self, mem),
            0xA3 => Lax::run_with(AddressingMode::IndirectIndexed, self, mem),
            0xB3 => Lax::run_with(AddressingMode::IndexedIndirect, self, mem),

            0x85 => Sta::run_with(AddressingMode::ZeroPage, self, mem),
            0x95 => Sta::run_with(AddressingMode::ZeroPageX, self, mem),
            0x8D => Sta::run_with(AddressingMode::Absolute, self, mem),
            0x9D => Sta::run_with(AddressingMode::AbsoluteX, self, mem),
            0x99 => Sta::run_with(AddressingMode::AbsoluteY, self, mem),
            0x81 => Sta::run_with(AddressingMode::IndirectIndexed, self, mem),
            0x91 => Sta::run_with(AddressingMode::IndexedIndirect, self, mem),

            0x86 => Stx::run_with(AddressingMode::ZeroPage, self, mem),
            0x96 => Stx::run_with(AddressingMode::ZeroPageY, self, mem),
            0x8E => Stx::run_with(AddressingMode::Absolute, self, mem),

            0x84 => Sty::run_with(AddressingMode::ZeroPage, self, mem),
            0x94 => Sty::run_with(AddressingMode::ZeroPageX, self, mem),
            0x8C => Sty::run_with(AddressingMode::Absolute, self, mem),

            0x87 => Sax::run_with(AddressingMode::ZeroPage, self, mem),
            0x97 => Sax::run_with(AddressingMode::ZeroPageY, self, mem),
            0x8F => Sax::run_with(AddressingMode::Absolute, self, mem),
            0x83 => Sax::run_with(AddressingMode::IndirectIndexed, self, mem),

            0xAA => Tax::run_with(AddressingMode::Implicit, self, mem),

            0xA8 => Tay::run_with(AddressingMode::Implicit, self, mem),

            0x8A => Txa::run_with(AddressingMode::Implicit, self, mem),

            0x98 => Tya::run_with(AddressingMode::Implicit, self, mem),

            0xBA => Tsx::run_with(AddressingMode::Implicit, self, mem),

            0x9A => Txs::run_with(AddressingMode::Implicit, self, mem),

            0x48 => Pha::run_with(AddressingMode::Implicit, self, mem),

            0x08 => Php::run_with(AddressingMode::Implicit, self, mem),

            0x68 => Pla::run_with(AddressingMode::Implicit, self, mem),

            0x28 => Plp::run_with(AddressingMode::Implicit, self, mem),

            0x29 => And::run_with(AddressingMode::Immediate, self, mem),
            0x25 => And::run_with(AddressingMode::ZeroPage, self, mem),
            0x35 => And::run_with(AddressingMode::ZeroPageX, self, mem),
            0x2D => And::run_with(AddressingMode::Absolute, self, mem),
            0x3D => And::run_with(AddressingMode::AbsoluteX, self, mem),
            0x39 => And::run_with(AddressingMode::AbsoluteY, self, mem),
            0x21 => And::run_with(AddressingMode::IndirectIndexed, self, mem),
            0x31 => And::run_with(AddressingMode::IndexedIndirect, self, mem),

            0x49 => Eor::run_with(AddressingMode::Immediate, self, mem),
            0x45 => Eor::run_with(AddressingMode::ZeroPage, self, mem),
            0x55 => Eor::run_with(AddressingMode::ZeroPageX, self, mem),
            0x4D => Eor::run_with(AddressingMode::Absolute, self, mem),
            0x5D => Eor::run_with(AddressingMode::AbsoluteX, self, mem),
            0x59 => Eor::run_with(AddressingMode::AbsoluteY, self, mem),
            0x41 => Eor::run_with(AddressingMode::IndirectIndexed, self, mem),
            0x51 => Eor::run_with(AddressingMode::IndexedIndirect, self, mem),

            0x09 => Ora::run_with(AddressingMode::Immediate, self, mem),
            0x05 => Ora::run_with(AddressingMode::ZeroPage, self, mem),
            0x15 => Ora::run_with(AddressingMode::ZeroPageX, self, mem),
            0x0D => Ora::run_with(AddressingMode::Absolute, self, mem),
            0x1D => Ora::run_with(AddressingMode::AbsoluteX, self, mem),
            0x19 => Ora::run_with(AddressingMode::AbsoluteY, self, mem),
            0x01 => Ora::run_with(AddressingMode::IndirectIndexed, self, mem),
            0x11 => Ora::run_with(AddressingMode::IndexedIndirect, self, mem),

            0x24 => Bit::run_with(AddressingMode::ZeroPage, self, mem),
            0x2C => Bit::run_with(AddressingMode::Absolute, self, mem),

            0x4C => Jmp::run_with(AddressingMode::Absolute, self, mem),
            0x6C => Jmp::run_with(AddressingMode::Indirect, self, mem),

            0x20 => Jsr::run_with(AddressingMode::Absolute, self, mem),

            0x60 => Rts::run_with(AddressingMode::Implicit, self, mem),

            0xD0 => Bne::run_with(AddressingMode::Relative, self, mem),
            0xF0 => Beq::run_with(AddressingMode::Relative, self, mem),
            0x10 => Bpl::run_with(AddressingMode::Relative, self, mem),
            0x90 => Bcc::run_with(AddressingMode::Relative, self, mem),
            0xB0 => Bcs::run_with(AddressingMode::Relative, self, mem),
            0x30 => Bmi::run_with(AddressingMode::Relative, self, mem),
            0x50 => Bvc::run_with(AddressingMode::Relative, self, mem),
            0x70 => Bvs::run_with(AddressingMode::Relative, self, mem),

            0xCA => Dex::run_with(AddressingMode::Implicit, self, mem),
            0x88 => Dey::run_with(AddressingMode::Implicit, self, mem),

            0xE8 => Incx::run_with(AddressingMode::Implicit, self, mem),
            0xC8 => Incy::run_with(AddressingMode::Implicit, self, mem),

            0x0A => Asl::run_with(AddressingMode::Accumulator, self, mem),
            0x06 => AslAddr::run_with(AddressingMode::ZeroPage, self, mem),
            0x16 => AslAddr::run_with(AddressingMode::ZeroPageX, self, mem),
            0x0E => AslAddr::run_with(AddressingMode::Absolute, self, mem),
            0x1E => AslAddr::run_with(AddressingMode::AbsoluteX, self, mem),

            0x07 => Slo::run_with(AddressingMode::ZeroPage, self, mem),
            0x17 => Slo::run_with(AddressingMode::ZeroPageX, self, mem),
            0x0F => Slo::run_with(AddressingMode::Absolute, self, mem),
            0x1F => Slo::run_with(AddressingMode::AbsoluteX, self, mem),
            0x1B => Slo::run_with(AddressingMode::AbsoluteY, self, mem),
            0x03 => Slo::run_with(AddressingMode::IndirectIndexed, self, mem),
            0x13 => Slo::run_with(AddressingMode::IndexedIndirect, self, mem),

            0x4A => Lsr::run_with(AddressingMode::Accumulator, self, mem),
            0x46 => LsrAddr::run_with(AddressingMode::ZeroPage, self, mem),
            0x56 => LsrAddr::run_with(AddressingMode::ZeroPageX, self, mem),
            0x4E => LsrAddr::run_with(AddressingMode::Absolute, self, mem),
            0x5E => LsrAddr::run_with(AddressingMode::AbsoluteX, self, mem),

            0x47 => Sre::run_with(AddressingMode::ZeroPage, self, mem),
            0x57 => Sre::run_with(AddressingMode::ZeroPageX, self, mem),
            0x4F => Sre::run_with(AddressingMode::Absolute, self, mem),
            0x5F => Sre::run_with(AddressingMode::AbsoluteX, self, mem),
            0x5B => Sre::run_with(AddressingMode::AbsoluteY, self, mem),
            0x43 => Sre::run_with(AddressingMode::IndirectIndexed, self, mem),
            0x53 => Sre::run_with(AddressingMode::IndexedIndirect, self, mem),

            0x2A => Rol::run_with(AddressingMode::Accumulator, self, mem),
            0x26 => RolAddr::run_with(AddressingMode::ZeroPage, self, mem),
            0x36 => RolAddr::run_with(AddressingMode::ZeroPageX, self, mem),
            0x2E => RolAddr::run_with(AddressingMode::Absolute, self, mem),
            0x3E => RolAddr::run_with(AddressingMode::AbsoluteX, self, mem),

            0x27 => Rla::run_with(AddressingMode::ZeroPage, self, mem),
            0x37 => Rla::run_with(AddressingMode::ZeroPageX, self, mem),
            0x2F => Rla::run_with(AddressingMode::Absolute, self, mem),
            0x3F => Rla::run_with(AddressingMode::AbsoluteX, self, mem),
            0x3B => Rla::run_with(AddressingMode::AbsoluteY, self, mem),
            0x23 => Rla::run_with(AddressingMode::IndirectIndexed, self, mem),
            0x33 => Rla::run_with(AddressingMode::IndexedIndirect, self, mem),

            0x6A => Ror::run_with(AddressingMode::Accumulator, self, mem),
            0x66 => RorAddr::run_with(AddressingMode::ZeroPage, self, mem),
            0x76 => RorAddr::run_with(AddressingMode::ZeroPageX, self, mem),
            0x6E => RorAddr::run_with(AddressingMode::Absolute, self, mem),
            0x7E => RorAddr::run_with(AddressingMode::AbsoluteX, self, mem),

            0x67 => Rra::run_with(AddressingMode::ZeroPage, self, mem),
            0x77 => Rra::run_with(AddressingMode::ZeroPageX, self, mem),
            0x6F => Rra::run_with(AddressingMode::Absolute, self, mem),
            0x7F => Rra::run_with(AddressingMode::AbsoluteX, self, mem),
            0x7B => Rra::run_with(AddressingMode::AbsoluteY, self, mem),
            0x63 => Rra::run_with(AddressingMode::IndirectIndexed, self, mem),
            0x73 => Rra::run_with(AddressingMode::IndexedIndirect, self, mem),

            0x18 => Clc::run_with(AddressingMode::Implicit, self, mem),
            0x38 => Sec::run_with(AddressingMode::Implicit, self, mem),

            0xD8 => Cld::run_with(AddressingMode::Implicit, self, mem),
            0xF8 => Sed::run_with(AddressingMode::Implicit, self, mem),

            0x58 => Cli::run_with(AddressingMode::Implicit, self, mem),
            0x78 => Sei::run_with(AddressingMode::Implicit, self, mem),

            0xB8 => Clv::run_with(AddressingMode::Implicit, self, mem),

            0xC9 => Cmp::run_with(AddressingMode::Immediate, self, mem),
            0xC5 => Cmp::run_with(AddressingMode::ZeroPage, self, mem),
            0xD5 => Cmp::run_with(AddressingMode::ZeroPageX, self, mem),
            0xCD => Cmp::run_with(AddressingMode::Absolute, self, mem),
            0xDD => Cmp::run_with(AddressingMode::AbsoluteX, self, mem),
            0xD9 => Cmp::run_with(AddressingMode::AbsoluteY, self, mem),
            0xC1 => Cmp::run_with(AddressingMode::IndirectIndexed, self, mem),
            0xD1 => Cmp::run_with(AddressingMode::IndexedIndirect, self, mem),

            0xE0 => Cpx::run_with(AddressingMode::Immediate, self, mem),
            0xE4 => Cpx::run_with(AddressingMode::ZeroPage, self, mem),
            0xEC => Cpx::run_with(AddressingMode::Absolute, self, mem),

            0xC0 => Cpy::run_with(AddressingMode::Immediate, self, mem),
            0xC4 => Cpy::run_with(AddressingMode::ZeroPage, self, mem),
            0xCC => Cpy::run_with(AddressingMode::Absolute, self, mem),

            0x69 => Adc::run_with(AddressingMode::Immediate, self, mem),
            0x65 => Adc::run_with(AddressingMode::ZeroPage, self, mem),
            0x75 => Adc::run_with(AddressingMode::ZeroPageX, self, mem),
            0x6D => Adc::run_with(AddressingMode::Absolute, self, mem),
            0x7D => Adc::run_with(AddressingMode::AbsoluteX, self, mem),
            0x79 => Adc::run_with(AddressingMode::AbsoluteY, self, mem),
            0x61 => Adc::run_with(AddressingMode::IndirectIndexed, self, mem),
            0x71 => Adc::run_with(AddressingMode::IndexedIndirect, self, mem),

            0xE9 => Sbc::run_with(AddressingMode::Immediate, self, mem),
            0xEB => Sbc::run_with(AddressingMode::Immediate, self, mem),
            0xE5 => Sbc::run_with(AddressingMode::ZeroPage, self, mem),
            0xF5 => Sbc::run_with(AddressingMode::ZeroPageX, self, mem),
            0xED => Sbc::run_with(AddressingMode::Absolute, self, mem),
            0xFD => Sbc::run_with(AddressingMode::AbsoluteX, self, mem),
            0xF9 => Sbc::run_with(AddressingMode::AbsoluteY, self, mem),
            0xE1 => Sbc::run_with(AddressingMode::IndirectIndexed, self, mem),
            0xF1 => Sbc::run_with(AddressingMode::IndexedIndirect, self, mem),

            0x00 => Brk::run_with(AddressingMode::Implicit, self, mem),

            0x40 => Rti::run_with(AddressingMode::Implicit, self, mem),

            0xE6 => Inc::run_with(AddressingMode::ZeroPage, self, mem),
            0xF6 => Inc::run_with(AddressingMode::ZeroPageX, self, mem),
            0xEE => Inc::run_with(AddressingMode::Absolute, self, mem),
            0xFE => Inc::run_with(AddressingMode::AbsoluteX, self, mem),

            0xE7 => Isc::run_with(AddressingMode::ZeroPage, self, mem),
            0xF7 => Isc::run_with(AddressingMode::ZeroPageX, self, mem),
            0xEF => Isc::run_with(AddressingMode::Absolute, self, mem),
            0xFF => Isc::run_with(AddressingMode::AbsoluteX, self, mem),
            0xFB => Isc::run_with(AddressingMode::AbsoluteY, self, mem),
            0xE3 => Isc::run_with(AddressingMode::IndirectIndexed, self, mem),
            0xF3 => Isc::run_with(AddressingMode::IndexedIndirect, self, mem),

            0xC6 => Dec::run_with(AddressingMode::ZeroPage, self, mem),
            0xD6 => Dec::run_with(AddressingMode::ZeroPageX, self, mem),
            0xCE => Dec::run_with(AddressingMode::Absolute, self, mem),
            0xDE => Dec::run_with(AddressingMode::AbsoluteX, self, mem),

            0xC7 => Dcp::run_with(AddressingMode::ZeroPage, self, mem),
            0xD7 => Dcp::run_with(AddressingMode::ZeroPageX, self, mem),
            0xCF => Dcp::run_with(AddressingMode::Absolute, self, mem),
            0xDF => Dcp::run_with(AddressingMode::AbsoluteX, self, mem),
            0xDB => Dcp::run_with(AddressingMode::AbsoluteY, self, mem),
            0xC3 => Dcp::run_with(AddressingMode::IndirectIndexed, self, mem),
            0xD3 => Dcp::run_with(AddressingMode::IndexedIndirect, self, mem),

            _ => unimplemented!("{:#04X} opcode not implemented yet!\n", opcode),
        };

        self.cycles += Wrapping(instr.cycles as usize);
        return instr;
    }

    fn imm(&mut self, _: &mut Asc) -> u16 {
        self.pc += 1;
        self.pc
    }

    fn zp(&mut self, ram: &mut Asc) -> u16 {
        self.pc += 1;
        ram.read(self.pc) as u16
    }

    fn zpx(&mut self, ram: &mut Asc) -> u16 {
        self.pc += 1;
        (ram.read(self.pc) as u16).wrapping_add(self.x as u16) & 0xff
    }

    fn zpy(&mut self, ram: &mut Asc) -> u16 {
        self.pc += 1;
        (ram.read(self.pc) as u16).wrapping_add(self.y as u16) & 0xff
    }

    fn abs(&mut self, ram: &mut Asc) -> u16 {
        self.pc += 1;
        let addr = ram.read(self.pc);
        self.pc += 1;
        (ram.read(self.pc) as u16) << 8 | addr as u16
    }

    fn abx(&mut self, ram: &mut Asc) -> u16 {
        self.pc += 1;
        let mut addr = ram.read(self.pc) as u16;
        self.pc += 1;
        addr |= (ram.read(self.pc) as u16) << 8;
        addr.wrapping_add(self.x as u16)
    }

    fn aby(&mut self, ram: &mut Asc) -> u16 {
        self.pc += 1;
        let mut addr = ram.read(self.pc) as u16;
        self.pc += 1;
        addr |= (ram.read(self.pc) as u16) << 8;
        addr.wrapping_add(self.y as u16)
    }

    fn inx(&mut self, ram: &mut Asc) -> u16 {
        self.pc += 1;
        let mut addr: u16 = ram.read(self.pc) as u16;
        addr = (addr.wrapping_add(self.x as u16) & 0xff) as u16;
        (ram.read(addr + 1) as u16) << 8 | ram.read(addr.into()) as u16
    }

    fn iny(&mut self, ram: &mut Asc) -> u16 {
        self.pc += 1;
        let addr: u16 = ram.read(self.pc) as u16;
        let addr = (ram.read(addr.wrapping_add(1)) as u16) << 8 | ram.read(addr) as u16;
        addr.wrapping_add(self.y as u16)
    }

    fn ind(&mut self, ram: &mut Asc) -> u16 {
        self.pc += 1;
        let addr = ram.read(self.pc);
        self.pc += 1;
        let addr = (ram.read(self.pc) as u16) << 8 | addr as u16;
        (ram.read(addr + 1) as u16) << 8 | ram.read(addr.into()) as u16
    }

    fn push(&mut self, value: u8, ram: &mut Asc) {
        ram.write(0x0100 | self.sp as u16, value);

        self.sp = self.sp.wrapping_sub(1);
    }

    fn pop(&mut self, ram: &mut Asc) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        ram.read(0x0100 | self.sp as u16)
    }

    fn push_long(&mut self, value: u16, ram: &mut Asc) {
        self.push(((value >> 8) & 0xff).try_into().unwrap(), ram);
        self.push((value & 0xff).try_into().unwrap(), ram);
    }

    fn pop_long(&mut self, ram: &mut Asc) -> u16 {
        let mut addr = self.pop(ram) as u16;
        addr |= (self.pop(ram) as u16) << 8;
        return addr;
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

    fn shift_left(&mut self, mut value: u8) -> u8 {
        self.carry_flag = value & NEGATIVE_MASK != 0;
        value <<= 1;
        self.negative_flag = value & NEGATIVE_MASK != 0;
        self.zero_flag = value == 0;

        self.pc += 1;
        return value;
    }

    fn shift_right(&mut self, mut value: u8) -> u8 {
        self.carry_flag = value & 1 != 0;
        value >>= 1;
        self.negative_flag = value & NEGATIVE_MASK != 0;
        self.zero_flag = value == 0;

        self.pc += 1;
        return value;
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
}
