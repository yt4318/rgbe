//! Instruction Execution
//!
//! This module implements the execution of all CPU instructions.

use crate::bus::MemoryBus;
use crate::common::{Byte, Word};
use super::instructions::{
    AddressingMode, ConditionType, Instruction, InstructionType, RegisterType,
    CB_INSTRUCTIONS,
};
use super::Cpu;

impl Cpu {
    /// Check if a register type is 16-bit
    fn is_16bit_reg(reg: RegisterType) -> bool {
        matches!(reg, RegisterType::Af | RegisterType::Bc | RegisterType::De | 
                      RegisterType::Hl | RegisterType::Sp | RegisterType::Pc)
    }

    /// Check condition for conditional instructions
    fn check_condition(&self, cond: ConditionType) -> bool {
        match cond {
            ConditionType::None => true,
            ConditionType::Z => self.regs.flag_z(),
            ConditionType::Nz => !self.regs.flag_z(),
            ConditionType::C => self.regs.flag_c(),
            ConditionType::Nc => !self.regs.flag_c(),
        }
    }

    /// Read from an 8-bit register
    fn read_reg8(&self, reg: RegisterType) -> Byte {
        match reg {
            RegisterType::A => self.regs.a,
            RegisterType::F => self.regs.f,
            RegisterType::B => self.regs.b,
            RegisterType::C => self.regs.c,
            RegisterType::D => self.regs.d,
            RegisterType::E => self.regs.e,
            RegisterType::H => self.regs.h,
            RegisterType::L => self.regs.l,
            _ => 0,
        }
    }

    /// Write to an 8-bit register
    fn write_reg8(&mut self, reg: RegisterType, value: Byte) {
        match reg {
            RegisterType::A => self.regs.a = value,
            RegisterType::F => self.regs.f = value & 0xF0,
            RegisterType::B => self.regs.b = value,
            RegisterType::C => self.regs.c = value,
            RegisterType::D => self.regs.d = value,
            RegisterType::E => self.regs.e = value,
            RegisterType::H => self.regs.h = value,
            RegisterType::L => self.regs.l = value,
            _ => {}
        }
    }

    /// Execute the current instruction
    pub fn execute<B: MemoryBus>(&mut self, bus: &mut B) {
        let inst = match self.current_instruction() {
            Some(i) => i,
            None => return,
        };

        match inst.inst_type {
            InstructionType::None => self.proc_none(),
            InstructionType::Nop => self.proc_nop(),
            InstructionType::Ld => self.proc_ld(bus, inst),
            InstructionType::Ldh => self.proc_ldh(bus, inst),
            InstructionType::Inc => self.proc_inc(bus, inst),
            InstructionType::Dec => self.proc_dec(bus, inst),
            InstructionType::Add => self.proc_add(inst),
            InstructionType::Adc => self.proc_adc(),
            InstructionType::Sub => self.proc_sub(inst),
            InstructionType::Sbc => self.proc_sbc(inst),
            InstructionType::And => self.proc_and(),
            InstructionType::Xor => self.proc_xor(),
            InstructionType::Or => self.proc_or(),
            InstructionType::Cp => self.proc_cp(),
            InstructionType::Jr => self.proc_jr(inst),
            InstructionType::Jp => self.proc_jp(inst),
            InstructionType::Call => self.proc_call(bus, inst),
            InstructionType::Ret => self.proc_ret(bus, inst),
            InstructionType::Reti => self.proc_reti(bus),
            InstructionType::Rst => self.proc_rst(bus, inst),
            InstructionType::Pop => self.proc_pop(bus, inst),
            InstructionType::Push => self.proc_push(bus, inst),
            InstructionType::Rlca => self.proc_rlca(),
            InstructionType::Rrca => self.proc_rrca(),
            InstructionType::Rla => self.proc_rla(),
            InstructionType::Rra => self.proc_rra(),
            InstructionType::Stop => self.proc_stop(),
            InstructionType::Halt => self.proc_halt(),
            InstructionType::Daa => self.proc_daa(),
            InstructionType::Cpl => self.proc_cpl(),
            InstructionType::Scf => self.proc_scf(),
            InstructionType::Ccf => self.proc_ccf(),
            InstructionType::Di => self.proc_di(),
            InstructionType::Ei => self.proc_ei(),
            InstructionType::Cb => self.proc_cb(bus),
            // CB-prefixed instructions (handled via proc_cb)
            InstructionType::Rlc | InstructionType::Rrc |
            InstructionType::Rl | InstructionType::Rr |
            InstructionType::Sla | InstructionType::Sra |
            InstructionType::Swap | InstructionType::Srl |
            InstructionType::Bit | InstructionType::Res |
            InstructionType::Set => {
                // These are handled by proc_cb
            }
        }
    }


    // ========== Instruction Processors ==========

    fn proc_none(&self) {
        panic!("INVALID INSTRUCTION!");
    }

    fn proc_nop(&self) {
        // Do nothing
    }

    fn proc_ld<B: MemoryBus>(&mut self, bus: &mut B, inst: &Instruction) {
        if self.dest_is_mem {
            if Self::is_16bit_reg(inst.reg2) {
                self.add_m_cycles(1);
                bus.write16(self.mem_dest, self.fetched_data);
            } else {
                bus.write(self.mem_dest, self.fetched_data as Byte);
            }
            self.add_m_cycles(1);
            return;
        }

        if inst.mode == AddressingMode::HlSpr {
            let hflag = (self.read_reg(inst.reg2) & 0xF) + (self.fetched_data & 0xF) >= 0x10;
            let cflag = (self.read_reg(inst.reg2) & 0xFF) + (self.fetched_data & 0xFF) >= 0x100;
            self.regs.set_flags(false, false, hflag, cflag);
            self.write_reg(inst.reg1, self.read_reg(inst.reg2).wrapping_add(self.fetched_data as i8 as i16 as Word));
            return;
        }

        self.write_reg(inst.reg1, self.fetched_data);
    }

    fn proc_ldh<B: MemoryBus>(&mut self, bus: &mut B, inst: &Instruction) {
        if inst.reg1 == RegisterType::A {
            self.regs.a = bus.read(0xFF00 | self.fetched_data);
        } else {
            bus.write(self.mem_dest, self.regs.a);
        }
        self.add_m_cycles(1);
    }

    fn proc_inc<B: MemoryBus>(&mut self, bus: &mut B, inst: &Instruction) {
        let mut val = self.read_reg(inst.reg1).wrapping_add(1);

        if Self::is_16bit_reg(inst.reg1) {
            self.add_m_cycles(1);
        }

        if inst.reg1 == RegisterType::Hl && inst.mode == AddressingMode::MemoryRegisterOnly {
            val = (bus.read(self.regs.hl()) as Word).wrapping_add(1) & 0xFF;
            bus.write(self.regs.hl(), val as Byte);
        } else {
            self.write_reg(inst.reg1, val);
            val = self.read_reg(inst.reg1);
        }

        // 16-bit INC doesn't affect flags (opcode & 0x03 == 0x03)
        if (self.cur_opcode & 0x03) == 0x03 {
            return;
        }

        self.regs.set_flag_z((val & 0xFF) == 0);
        self.regs.set_flag_n(false);
        self.regs.set_flag_h((val & 0x0F) == 0);
    }

    fn proc_dec<B: MemoryBus>(&mut self, bus: &mut B, inst: &Instruction) {
        let mut val = self.read_reg(inst.reg1).wrapping_sub(1);

        if Self::is_16bit_reg(inst.reg1) {
            self.add_m_cycles(1);
        }

        if inst.reg1 == RegisterType::Hl && inst.mode == AddressingMode::MemoryRegisterOnly {
            val = (bus.read(self.regs.hl()) as Word).wrapping_sub(1) & 0xFF;
            bus.write(self.regs.hl(), val as Byte);
        } else {
            self.write_reg(inst.reg1, val);
            val = self.read_reg(inst.reg1);
        }

        // 16-bit DEC doesn't affect flags (opcode & 0x0B == 0x0B)
        if (self.cur_opcode & 0x0B) == 0x0B {
            return;
        }

        self.regs.set_flag_z((val & 0xFF) == 0);
        self.regs.set_flag_n(true);
        self.regs.set_flag_h((val & 0x0F) == 0x0F);
    }

    fn proc_add(&mut self, inst: &Instruction) {
        let reg_val = self.read_reg(inst.reg1);
        let mut val = reg_val.wrapping_add(self.fetched_data);
        let is_16bit = Self::is_16bit_reg(inst.reg1);

        if is_16bit {
            self.add_m_cycles(1);
        }

        if inst.reg1 == RegisterType::Sp {
            val = reg_val.wrapping_add(self.fetched_data as i8 as i16 as Word);
        }

        let mut z = (val & 0xFF) == 0;
        let mut h = (reg_val & 0xF) + (self.fetched_data & 0xF) >= 0x10;
        let mut c = (reg_val & 0xFF) + (self.fetched_data & 0xFF) >= 0x100;

        if is_16bit {
            z = self.regs.flag_z(); // Z unchanged for 16-bit ADD
            h = (reg_val & 0xFFF) + (self.fetched_data & 0xFFF) >= 0x1000;
            let n = (reg_val as u32) + (self.fetched_data as u32);
            c = n >= 0x10000;
        }

        if inst.reg1 == RegisterType::Sp {
            z = false;
            h = (reg_val & 0xF) + (self.fetched_data & 0xF) >= 0x10;
            c = (reg_val & 0xFF) + (self.fetched_data & 0xFF) >= 0x100;
        }

        self.write_reg(inst.reg1, val);
        self.regs.set_flags(z, false, h, c);
    }

    fn proc_adc(&mut self) {
        let u = self.fetched_data;
        let a = self.regs.a as Word;
        let c = if self.regs.flag_c() { 1 } else { 0 };

        self.regs.a = ((a + u + c) & 0xFF) as Byte;

        self.regs.set_flags(
            self.regs.a == 0,
            false,
            (a & 0xF) + (u & 0xF) + c > 0xF,
            a + u + c > 0xFF,
        );
    }

    fn proc_sub(&mut self, inst: &Instruction) {
        let reg_val = self.read_reg(inst.reg1);
        let val = reg_val.wrapping_sub(self.fetched_data);

        let z = (val & 0xFF) == 0;
        let h = (reg_val as i32 & 0xF) - (self.fetched_data as i32 & 0xF) < 0;
        let c = (reg_val as i32) - (self.fetched_data as i32) < 0;

        self.write_reg(inst.reg1, val);
        self.regs.set_flags(z, true, h, c);
    }

    fn proc_sbc(&mut self, inst: &Instruction) {
        let c_flag = if self.regs.flag_c() { 1u16 } else { 0 };
        let val = self.fetched_data.wrapping_add(c_flag);
        let reg_val = self.read_reg(inst.reg1);

        let z = reg_val.wrapping_sub(val) == 0;
        let h = (reg_val as i32 & 0xF) - (self.fetched_data as i32 & 0xF) - (c_flag as i32) < 0;
        let c = (reg_val as i32) - (self.fetched_data as i32) - (c_flag as i32) < 0;

        self.write_reg(inst.reg1, reg_val.wrapping_sub(val));
        self.regs.set_flags(z, true, h, c);
    }


    fn proc_and(&mut self) {
        self.regs.a &= self.fetched_data as Byte;
        self.regs.set_flags(self.regs.a == 0, false, true, false);
    }

    fn proc_xor(&mut self) {
        self.regs.a ^= (self.fetched_data & 0xFF) as Byte;
        self.regs.set_flags(self.regs.a == 0, false, false, false);
    }

    fn proc_or(&mut self) {
        self.regs.a |= (self.fetched_data & 0xFF) as Byte;
        self.regs.set_flags(self.regs.a == 0, false, false, false);
    }

    fn proc_cp(&mut self) {
        let n = (self.regs.a as i32) - (self.fetched_data as i32);
        self.regs.set_flags(
            n == 0,
            true,
            (self.regs.a as i32 & 0x0F) - (self.fetched_data as i32 & 0x0F) < 0,
            n < 0,
        );
    }

    fn proc_jr(&mut self, inst: &Instruction) {
        let rel = (self.fetched_data & 0xFF) as i8;
        let addr = self.regs.pc.wrapping_add(rel as i16 as Word);
        self.jump_to_if(addr, inst.cond);
    }

    fn proc_jp(&mut self, inst: &Instruction) {
        self.jump_to_if(self.fetched_data, inst.cond);
    }

    fn proc_call<B: MemoryBus>(&mut self, bus: &mut B, inst: &Instruction) {
        if self.check_condition(inst.cond) {
            self.add_m_cycles(2);
            self.stack_push16(bus, self.regs.pc);
            self.regs.pc = self.fetched_data;
            self.add_m_cycles(1);
        }
    }

    fn proc_ret<B: MemoryBus>(&mut self, bus: &mut B, inst: &Instruction) {
        if inst.cond != ConditionType::None {
            self.add_m_cycles(1);
        }
        if self.check_condition(inst.cond) {
            self.regs.pc = self.stack_pop16(bus);
            self.add_m_cycles(3);
        }
    }

    fn proc_reti<B: MemoryBus>(&mut self, bus: &mut B) {
        self.ime = true;
        self.regs.pc = self.stack_pop16(bus);
        self.add_m_cycles(3);
    }

    fn proc_rst<B: MemoryBus>(&mut self, bus: &mut B, inst: &Instruction) {
        self.add_m_cycles(2);
        self.stack_push16(bus, self.regs.pc);
        self.regs.pc = inst.param as Word;
        self.add_m_cycles(1);
    }

    fn proc_pop<B: MemoryBus>(&mut self, bus: &mut B, inst: &Instruction) {
        let val = self.stack_pop16(bus);
        self.add_m_cycles(2);
        self.write_reg(inst.reg1, val);
        
        // AF special case: lower 4 bits of F are always 0
        if inst.reg1 == RegisterType::Af {
            self.regs.f &= 0xF0;
        }
    }

    fn proc_push<B: MemoryBus>(&mut self, bus: &mut B, inst: &Instruction) {
        let val = self.read_reg(inst.reg1);
        self.stack_push16(bus, val);
        self.add_m_cycles(3);
    }

    fn proc_rlca(&mut self) {
        let u = self.regs.a;
        let c = (u >> 7) & 1;
        self.regs.a = (u << 1) | c;
        self.regs.set_flags(false, false, false, c != 0);
    }

    fn proc_rrca(&mut self) {
        let b = self.regs.a & 1;
        self.regs.a = (self.regs.a >> 1) | (b << 7);
        self.regs.set_flags(false, false, false, b != 0);
    }

    fn proc_rla(&mut self) {
        let u = self.regs.a;
        let cf = if self.regs.flag_c() { 1 } else { 0 };
        let c = (u >> 7) & 1;
        self.regs.a = (u << 1) | cf;
        self.regs.set_flags(false, false, false, c != 0);
    }

    fn proc_rra(&mut self) {
        let carry = if self.regs.flag_c() { 1 } else { 0 };
        let new_c = self.regs.a & 1;
        self.regs.a = (self.regs.a >> 1) | (carry << 7);
        self.regs.set_flags(false, false, false, new_c != 0);
    }

    fn proc_stop(&mut self) {
        // STOP instruction - typically used for speed switching on CGB
        // For DMG, this just halts until a button is pressed
    }

    fn proc_halt(&mut self) {
        self.halted = true;
    }


    fn proc_daa(&mut self) {
        let mut u: u8 = 0;
        let mut fc = false;

        if self.regs.flag_h() || (!self.regs.flag_n() && (self.regs.a & 0xF) > 9) {
            u = 6;
        }

        if self.regs.flag_c() || (!self.regs.flag_n() && self.regs.a > 0x99) {
            u |= 0x60;
            fc = true;
        }

        if self.regs.flag_n() {
            self.regs.a = self.regs.a.wrapping_sub(u);
        } else {
            self.regs.a = self.regs.a.wrapping_add(u);
        }

        self.regs.set_flag_z(self.regs.a == 0);
        self.regs.set_flag_h(false);
        self.regs.set_flag_c(fc);
    }

    fn proc_cpl(&mut self) {
        self.regs.a = !self.regs.a;
        self.regs.set_flag_n(true);
        self.regs.set_flag_h(true);
    }

    fn proc_scf(&mut self) {
        self.regs.set_flag_n(false);
        self.regs.set_flag_h(false);
        self.regs.set_flag_c(true);
    }

    fn proc_ccf(&mut self) {
        self.regs.set_flag_n(false);
        self.regs.set_flag_h(false);
        self.regs.set_flag_c(!self.regs.flag_c());
    }

    fn proc_di(&mut self) {
        self.ime = false;
    }

    fn proc_ei(&mut self) {
        self.enabling_ime = true;
    }

    fn proc_cb<B: MemoryBus>(&mut self, bus: &mut B) {
        let op = self.fetched_data as Byte;
        let cb_inst = &CB_INSTRUCTIONS[op as usize];
        let reg = cb_inst.reg1;
        let bit = cb_inst.param;
        
        // Read register value (or memory for HL)
        let reg_val = if reg == RegisterType::Hl {
            bus.read(self.regs.hl())
        } else {
            self.read_reg8(reg)
        };

        // Decode CB operation type from opcode
        let bit_op = (op >> 6) & 0b11;

        // fetch_instruction + fetch_data already consumed 2 M-cycles for CB opcodes.
        // Additional cycles:
        // - register targets: +0
        // - BIT b,(HL): +1
        // - other (HL) operations: +2
        if reg == RegisterType::Hl {
            if bit_op == 1 {
                self.add_m_cycles(1);
            } else {
                self.add_m_cycles(2);
            }
        }

        match bit_op {
            1 => {
                // BIT
                self.regs.set_flag_z((reg_val & (1 << bit)) == 0);
                self.regs.set_flag_n(false);
                self.regs.set_flag_h(true);
                return;
            }
            2 => {
                // RES
                let result = reg_val & !(1 << bit);
                self.write_cb_result(bus, reg, result);
                return;
            }
            3 => {
                // SET
                let result = reg_val | (1 << bit);
                self.write_cb_result(bus, reg, result);
                return;
            }
            _ => {}
        }

        // Rotate/shift operations (bit_op == 0)
        let flag_c = self.regs.flag_c();
        let bit_idx = (op >> 3) & 0b111;

        let (result, set_c) = match bit_idx {
            0 => {
                // RLC
                let c = (reg_val >> 7) & 1;
                let r = (reg_val << 1) | c;
                (r, c != 0)
            }
            1 => {
                // RRC
                let c = reg_val & 1;
                let r = (reg_val >> 1) | (c << 7);
                (r, c != 0)
            }
            2 => {
                // RL
                let c = (reg_val >> 7) & 1;
                let r = (reg_val << 1) | (if flag_c { 1 } else { 0 });
                (r, c != 0)
            }
            3 => {
                // RR
                let c = reg_val & 1;
                let r = (reg_val >> 1) | (if flag_c { 0x80 } else { 0 });
                (r, c != 0)
            }
            4 => {
                // SLA
                let c = (reg_val >> 7) & 1;
                let r = reg_val << 1;
                (r, c != 0)
            }
            5 => {
                // SRA (arithmetic shift right - preserves sign bit)
                let c = reg_val & 1;
                let r = ((reg_val as i8) >> 1) as u8;
                (r, c != 0)
            }
            6 => {
                // SWAP
                let r = ((reg_val & 0xF0) >> 4) | ((reg_val & 0x0F) << 4);
                (r, false)
            }
            7 => {
                // SRL (logical shift right)
                let c = reg_val & 1;
                let r = reg_val >> 1;
                (r, c != 0)
            }
            _ => (reg_val, false),
        };

        self.write_cb_result(bus, reg, result);
        self.regs.set_flags(result == 0, false, false, set_c);
    }

    fn write_cb_result<B: MemoryBus>(&mut self, bus: &mut B, reg: RegisterType, value: Byte) {
        if reg == RegisterType::Hl {
            bus.write(self.regs.hl(), value);
        } else {
            self.write_reg8(reg, value);
        }
    }

    fn jump_to_if(&mut self, addr: Word, cond: ConditionType) {
        if self.check_condition(cond) {
            self.regs.pc = addr;
            self.add_m_cycles(1);
        }
    }

    // ========== Stack Operations ==========

    /// Push an 8-bit value onto the stack
    /// Decrements SP first, then writes the value
    pub fn stack_push8<B: MemoryBus>(&mut self, bus: &mut B, value: Byte) {
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        bus.write(self.regs.sp, value);
    }

    /// Pop an 8-bit value from the stack
    /// Reads the value first, then increments SP
    pub fn stack_pop8<B: MemoryBus>(&mut self, bus: &mut B) -> Byte {
        let value = bus.read(self.regs.sp);
        self.regs.sp = self.regs.sp.wrapping_add(1);
        value
    }

    /// Push a 16-bit value onto the stack
    /// High byte is pushed first, then low byte (SP ends up pointing to low byte)
    pub fn stack_push16<B: MemoryBus>(&mut self, bus: &mut B, value: Word) {
        let hi = ((value >> 8) & 0xFF) as Byte;
        let lo = (value & 0xFF) as Byte;
        self.stack_push8(bus, hi);
        self.stack_push8(bus, lo);
    }

    /// Pop a 16-bit value from the stack
    /// Low byte is popped first, then high byte
    pub fn stack_pop16<B: MemoryBus>(&mut self, bus: &mut B) -> Word {
        let lo = self.stack_pop8(bus) as Word;
        let hi = self.stack_pop8(bus) as Word;
        (hi << 8) | lo
    }

    /// Handle pending interrupts
    /// 
    /// Returns true if an interrupt was handled
    pub fn handle_interrupts<B: MemoryBus>(&mut self, bus: &mut B) -> bool {
        // Check if any interrupts are pending and enabled
        if !self.ime || !self.interrupts_pending() {
            return false;
        }

        // Get the highest priority pending interrupt
        if let Some(interrupt) = self.get_pending_interrupt() {
            // Disable IME
            self.ime = false;
            
            // Clear the interrupt flag
            self.clear_interrupt(interrupt);
            
            // Push PC to stack
            self.stack_push16(bus, self.regs.pc);
            
            // Jump to interrupt vector
            self.regs.pc = interrupt.vector();
            
            // Exit halt mode if halted
            self.halted = false;
            
            return true;
        }
        
        false
    }
}
