use super::interconnect::Interconnect;
use super::registers::{Registers, Reg8, Reg16, Flag};

use std::u8;
use std::u16;

pub struct Cpu<'a> {
    regs: Registers,
    interconnect: &'a mut Interconnect,
}

trait Src8 {
    fn read(self, cpu: &mut Cpu) -> u8;
}

trait Dst8 {
    fn write(self, cpu: &mut Cpu, value: u8);
}

trait Src16 {
    fn read(self, cpu: &mut Cpu) -> u16;
}

trait Dst16 {
    fn write(self, cpu: &mut Cpu, value: u16);
}

impl Src8 for Reg8 {
    fn read(self, cpu: &mut Cpu) -> u8 {
        cpu.regs.read_u8(self)
    }
}

impl Dst8 for Reg8 {
    fn write(self, cpu: &mut Cpu, value: u8) {
        cpu.regs.write_u8(self, value)
    }
}

impl Src16 for Reg16 {
    fn read(self, cpu: &mut Cpu) -> u16 {
        cpu.regs.read_u16(self)
    }
}

impl Dst16 for Reg16 {
    fn write(self, cpu: &mut Cpu, value: u16) {
        cpu.regs.write_u16(self, value)
    }
}

struct Immediate8;
struct Immediate16;

impl Src8 for Immediate8 {
    fn read(self, cpu: &mut Cpu) -> u8 {
        cpu.fetch_u8()
    }
}

impl Src16 for Immediate16 {
    fn read(self, cpu: &mut Cpu) -> u16 {
        cpu.fetch_u16()
    }
}

struct HiMem;

impl Src8 for HiMem {
    fn read(self, cpu: &mut Cpu) -> u8 {
        let offset = cpu.fetch_u8() as u16;
        let address = 0xff00 + offset;
        cpu.interconnect.read(address)
    }
}

impl Dst8 for HiMem {
    fn write(self, cpu: &mut Cpu, value: u8) {
        let offset = cpu.fetch_u8() as u16;
        let address = 0xff00 + offset;
        cpu.interconnect.write(address, value)
    }
}

enum Cond {
    Z,
    C,
    NZ,
    NC,
}

trait JumpCond {
    fn jump(self, cpu: &Cpu) -> bool;
}

impl JumpCond for Cond {
    fn jump(self, cpu: &Cpu) -> bool {
        use self::Cond::*;
        match self {
            Z => cpu.regs.is_flag_set(Flag::Z),
            C => cpu.regs.is_flag_set(Flag::C),
            NZ => !cpu.regs.is_flag_set(Flag::Z),
            NC => !cpu.regs.is_flag_set(Flag::C),
        }
    }
}

impl<'a> Cpu<'a> {
    pub fn new(interconnect: &'a mut Interconnect) -> Cpu {
        Cpu {
            regs: Registers::new(),
            interconnect: interconnect,
        }
    }

    pub fn execute_instruction(&mut self) {

        println!("0x{:x}", self.regs.pc);

        let opcode = self.fetch_u8();

        match opcode {
            // NOP
            0x00 => {}

            // JR NZ,r8
            0x20 => self.jr(Cond::NZ, Immediate8),

            // LD A,d8
            0x3e => self.load(Reg8::A, Immediate8),

            // XOR A
            0xaf => self.xor(Reg8::A),

            // JP a16
            0xc3 => self.jump(Immediate16),

            // CB PREFIX
            0xcb => self.execute_cb_instruction(),

            // LDH (a8),A
            0xe0 => self.load(HiMem, Reg8::A),

            // LDH A,(a8)
            0xf0 => self.load(Reg8::A, HiMem),

            // CP d8
            0xfe => self.compare(Immediate8),

            _ => panic!("Opcode not implemented: 0x{:x}", opcode),
        }

    }

    fn execute_cb_instruction(&mut self) {

        let opcode = self.fetch_u8();

        match opcode {

            // BIT 7,A
            0x7f => self.bit(7, Reg8::A),

            _ => panic!("CB opcode not implemented: 0x{:x}", opcode),
        }

    }

    fn load<D: Dst8, S: Src8>(&mut self, dst: D, src: S) {
        let value = src.read(self);
        dst.write(self, value)
    }

    fn jump<S: Src16>(&mut self, src: S) {
        let new_pc = src.read(self);
        self.regs.pc = new_pc
    }

    fn jr<C: JumpCond, S: Src8>(&mut self, cond: C, src: S) {
        let offset = src.read(self) as u16;
        if cond.jump(self) {
            if (offset & 0x80) != 0 {
                self.regs.pc = self.regs.pc.wrapping_sub(offset)
            } else {
                self.regs.pc = self.regs.pc.wrapping_add(offset)
            }
        }
    }

    fn bit<S: Src8>(&mut self, bit: u8, src: S) {
        let value = src.read(self) >> bit;
        self.regs.set_flag_value(Flag::Z, (value & 0x01) == 0);
        self.regs.clear_flag(Flag::N);
        self.regs.set_flag(Flag::H);
    }

    fn xor<S: Src8>(&mut self, src: S) {
        let value = src.read(self);
        let result = self.regs.a ^ value;
        self.regs.set_flag_value(Flag::Z, result == 0);
        self.regs.clear_flag(Flag::N);
        self.regs.clear_flag(Flag::H);
        self.regs.clear_flag(Flag::C);
        self.regs.a = result
    }

    fn compare<S: Src8>(&mut self, src: S) {
        let value = src.read(self);
        self.regs.set_flag(Flag::N);

        let carry = self.regs.a < value;
        self.regs.set_flag_value(Flag::C, carry);

        let zero = self.regs.a == value;
        self.regs.set_flag_value(Flag::Z, zero);

        let half_carry = (self.regs.a.wrapping_sub(value) & 0xf) > (self.regs.a & 0xf);
        self.regs.set_flag_value(Flag::H, half_carry)
    }

    fn fetch_u8(&mut self) -> u8 {
        let pc = self.regs.pc;
        let value = self.interconnect.read(pc);
        self.regs.pc = pc.wrapping_add(1);
        value
    }

    fn fetch_u16(&mut self) -> u16 {
        let low = self.fetch_u8() as u16;
        let high = self.fetch_u8() as u16;
        (high << 8) | low
    }
}
