//! Instruction Fetch
//!
//! This module implements instruction fetching and operand decoding.

use crate::bus::MemoryBus;
use crate::common::{Byte, Word};
use super::instructions::{
    AddressingMode, Instruction, RegisterType, instruction_by_opcode,
};
use super::Cpu;

impl Cpu {
    /// Read a value from a register
    pub fn read_reg(&self, reg: RegisterType) -> Word {
        match reg {
            RegisterType::None => 0,
            RegisterType::A => self.regs.a as Word,
            RegisterType::F => self.regs.f as Word,
            RegisterType::B => self.regs.b as Word,
            RegisterType::C => self.regs.c as Word,
            RegisterType::D => self.regs.d as Word,
            RegisterType::E => self.regs.e as Word,
            RegisterType::H => self.regs.h as Word,
            RegisterType::L => self.regs.l as Word,
            RegisterType::Af => self.regs.af(),
            RegisterType::Bc => self.regs.bc(),
            RegisterType::De => self.regs.de(),
            RegisterType::Hl => self.regs.hl(),
            RegisterType::Sp => self.regs.sp,
            RegisterType::Pc => self.regs.pc,
        }
    }

    /// Write a value to a register
    pub fn write_reg(&mut self, reg: RegisterType, value: Word) {
        match reg {
            RegisterType::None => {}
            RegisterType::A => self.regs.a = value as Byte,
            RegisterType::F => self.regs.f = (value & 0xF0) as Byte,
            RegisterType::B => self.regs.b = value as Byte,
            RegisterType::C => self.regs.c = value as Byte,
            RegisterType::D => self.regs.d = value as Byte,
            RegisterType::E => self.regs.e = value as Byte,
            RegisterType::H => self.regs.h = value as Byte,
            RegisterType::L => self.regs.l = value as Byte,
            RegisterType::Af => self.regs.set_af(value),
            RegisterType::Bc => self.regs.set_bc(value),
            RegisterType::De => self.regs.set_de(value),
            RegisterType::Hl => self.regs.set_hl(value),
            RegisterType::Sp => self.regs.sp = value,
            RegisterType::Pc => self.regs.pc = value,
        }
    }

    /// Fetch the next opcode and get the instruction
    pub fn fetch_instruction<B: MemoryBus>(&mut self, bus: &B) -> &'static Instruction {
        self.cur_opcode = bus.read(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        let inst = instruction_by_opcode(self.cur_opcode);
        self.set_current_instruction(Some(inst));
        inst
    }

    /// Fetch operand data based on addressing mode
    pub fn fetch_data<B: MemoryBus>(&mut self, bus: &B) {
        self.mem_dest = 0;
        self.dest_is_mem = false;

        let inst = match self.current_instruction() {
            Some(i) => i,
            None => return,
        };

        match inst.mode {
            AddressingMode::Implied => {}

            AddressingMode::Register => {
                self.fetched_data = self.read_reg(inst.reg1);
            }

            AddressingMode::RegisterRegister => {
                self.fetched_data = self.read_reg(inst.reg2);
            }

            AddressingMode::RegisterD8 | AddressingMode::D8 => {
                self.fetched_data = bus.read(self.regs.pc) as Word;
                self.regs.pc = self.regs.pc.wrapping_add(1);
            }

            AddressingMode::RegisterD16 | AddressingMode::D16 => {
                let lo = bus.read(self.regs.pc) as Word;
                let hi = bus.read(self.regs.pc.wrapping_add(1)) as Word;
                self.fetched_data = lo | (hi << 8);
                self.regs.pc = self.regs.pc.wrapping_add(2);
            }

            AddressingMode::MemoryRegister => {
                self.fetched_data = self.read_reg(inst.reg2);
                self.mem_dest = self.read_reg(inst.reg1);
                self.dest_is_mem = true;
                if inst.reg1 == RegisterType::C {
                    self.mem_dest |= 0xFF00;
                }
            }

            AddressingMode::RegisterMemory => {
                let mut addr = self.read_reg(inst.reg2);
                if inst.reg2 == RegisterType::C {
                    addr |= 0xFF00;
                }
                self.fetched_data = bus.read(addr) as Word;
            }

            AddressingMode::RegisterHli => {
                self.fetched_data = bus.read(self.read_reg(inst.reg2)) as Word;
                let hl = self.regs.hl().wrapping_add(1);
                self.regs.set_hl(hl);
            }

            AddressingMode::RegisterHld => {
                self.fetched_data = bus.read(self.read_reg(inst.reg2)) as Word;
                let hl = self.regs.hl().wrapping_sub(1);
                self.regs.set_hl(hl);
            }

            AddressingMode::HliRegister => {
                self.fetched_data = self.read_reg(inst.reg2);
                self.mem_dest = self.read_reg(inst.reg1);
                self.dest_is_mem = true;
                let hl = self.regs.hl().wrapping_add(1);
                self.regs.set_hl(hl);
            }

            AddressingMode::HldRegister => {
                self.fetched_data = self.read_reg(inst.reg2);
                self.mem_dest = self.read_reg(inst.reg1);
                self.dest_is_mem = true;
                let hl = self.regs.hl().wrapping_sub(1);
                self.regs.set_hl(hl);
            }

            AddressingMode::RegisterA8 => {
                self.fetched_data = bus.read(self.regs.pc) as Word;
                self.regs.pc = self.regs.pc.wrapping_add(1);
            }

            AddressingMode::A8Register => {
                self.mem_dest = (bus.read(self.regs.pc) as Word) | 0xFF00;
                self.dest_is_mem = true;
                self.regs.pc = self.regs.pc.wrapping_add(1);
            }

            AddressingMode::HlSpr => {
                self.fetched_data = bus.read(self.regs.pc) as Word;
                self.regs.pc = self.regs.pc.wrapping_add(1);
            }

            AddressingMode::A16Register => {
                let lo = bus.read(self.regs.pc) as Word;
                let hi = bus.read(self.regs.pc.wrapping_add(1)) as Word;
                self.mem_dest = lo | (hi << 8);
                self.dest_is_mem = true;
                self.regs.pc = self.regs.pc.wrapping_add(2);
                self.fetched_data = self.read_reg(inst.reg2);
            }

            AddressingMode::MemoryRegisterD8 => {
                self.fetched_data = bus.read(self.regs.pc) as Word;
                self.regs.pc = self.regs.pc.wrapping_add(1);
                self.mem_dest = self.read_reg(inst.reg1);
                self.dest_is_mem = true;
            }

            AddressingMode::MemoryRegisterOnly => {
                self.mem_dest = self.read_reg(inst.reg1);
                self.dest_is_mem = true;
                self.fetched_data = bus.read(self.read_reg(inst.reg1)) as Word;
            }

            AddressingMode::RegisterA16 => {
                let lo = bus.read(self.regs.pc) as Word;
                let hi = bus.read(self.regs.pc.wrapping_add(1)) as Word;
                let addr = lo | (hi << 8);
                self.regs.pc = self.regs.pc.wrapping_add(2);
                self.fetched_data = bus.read(addr) as Word;
            }
        }
    }
}
