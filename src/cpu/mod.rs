//! CPU Module
//!
//! This module implements the Sharp LR35902 CPU emulation for the Game Boy.

pub mod execute;
pub mod fetch;
pub mod instructions;
pub mod registers;

use crate::common::{Byte, Word};
use registers::Registers;
use std::fmt;

/// CPU state for the Sharp LR35902 processor
#[derive(Debug)]
pub struct Cpu {
    /// CPU registers (A, F, B, C, D, E, H, L, SP, PC)
    pub regs: Registers,
    /// CPU is in halted state (waiting for interrupt)
    pub halted: bool,
    /// Interrupt Master Enable flag
    pub ime: bool,
    /// IME will be enabled after next instruction (for EI instruction)
    pub enabling_ime: bool,
    /// Interrupt Enable register (0xFFFF)
    pub ie_register: Byte,
    /// Interrupt Flags register (0xFF0F)
    pub int_flags: Byte,
    /// Fetched data for current instruction
    pub fetched_data: Word,
    /// Memory destination address for current instruction
    pub mem_dest: Word,
    /// Whether destination is memory (vs register)
    pub dest_is_mem: bool,
    /// Current opcode being executed
    pub cur_opcode: Byte,
    /// Current instruction reference
    cur_inst: Option<&'static instructions::Instruction>,
    /// M-cycles consumed by the current step
    pending_m_cycles: u32,
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Cpu {
    /// Create a new CPU with default (zeroed) state
    pub fn new() -> Self {
        Self {
            regs: Registers::new(),
            halted: false,
            ime: false,
            enabling_ime: false,
            ie_register: 0,
            int_flags: 0,
            fetched_data: 0,
            mem_dest: 0,
            dest_is_mem: false,
            cur_opcode: 0,
            cur_inst: None,
            pending_m_cycles: 0,
        }
    }

    /// Initialize CPU to boot ROM skip state
    ///
    /// This sets the registers to the values they would have after
    /// the boot ROM has finished executing.
    pub fn init(&mut self) {
        // Boot ROM skip values (DMG)
        self.regs.pc = 0x0100; // Entry point after boot ROM
        self.regs.sp = 0xFFFE; // Stack pointer
        self.regs.set_af(0x01B0); // A=0x01, F=0xB0 (Z=1, N=0, H=1, C=1)
        self.regs.set_bc(0x0013);
        self.regs.set_de(0x00D8);
        self.regs.set_hl(0x014D);

        self.halted = false;
        self.ime = false;
        self.enabling_ime = false;
        self.ie_register = 0;
        self.int_flags = 0;
        self.pending_m_cycles = 0;
    }

    /// Get the current instruction being executed
    pub fn current_instruction(&self) -> Option<&'static instructions::Instruction> {
        self.cur_inst
    }

    /// Set the current instruction
    pub fn set_current_instruction(&mut self, inst: Option<&'static instructions::Instruction>) {
        self.cur_inst = inst;
    }

    /// Reset M-cycle accounting for a new CPU step
    pub fn reset_step_cycles(&mut self) {
        self.pending_m_cycles = 0;
    }

    /// Add consumed M-cycles
    pub fn add_m_cycles(&mut self, cycles: u32) {
        self.pending_m_cycles = self.pending_m_cycles.saturating_add(cycles);
    }

    /// Consume and return pending T-cycles (M-cycles * 4)
    pub fn take_t_cycles(&mut self) -> u32 {
        let t_cycles = self.pending_m_cycles.saturating_mul(4);
        self.pending_m_cycles = 0;
        t_cycles
    }

    /// Request an interrupt
    ///
    /// Sets the corresponding bit in the IF register
    pub fn request_interrupt(&mut self, interrupt: InterruptType) {
        self.int_flags |= interrupt.bit();
    }

    /// Check if any interrupts are pending and enabled
    pub fn interrupts_pending(&self) -> bool {
        (self.int_flags & self.ie_register & 0x1F) != 0
    }

    /// Get the highest priority pending interrupt
    pub fn get_pending_interrupt(&self) -> Option<InterruptType> {
        let pending = self.int_flags & self.ie_register & 0x1F;
        if pending == 0 {
            return None;
        }

        // Check in priority order (VBlank highest, Joypad lowest)
        for &int_type in InterruptType::all() {
            if (pending & int_type.bit()) != 0 {
                return Some(int_type);
            }
        }
        None
    }

    /// Clear an interrupt flag
    pub fn clear_interrupt(&mut self, interrupt: InterruptType) {
        self.int_flags &= !interrupt.bit();
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CPU: PC={:04X} SP={:04X} AF={:04X} BC={:04X} DE={:04X} HL={:04X} IME={} HALT={}",
            self.regs.pc,
            self.regs.sp,
            self.regs.af(),
            self.regs.bc(),
            self.regs.de(),
            self.regs.hl(),
            if self.ime { 1 } else { 0 },
            if self.halted { 1 } else { 0 }
        )
    }
}

/// Interrupt types supported by the Game Boy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptType {
    /// VBlank interrupt (highest priority)
    VBlank,
    /// LCD STAT interrupt
    LcdStat,
    /// Timer interrupt
    Timer,
    /// Serial interrupt
    Serial,
    /// Joypad interrupt (lowest priority)
    Joypad,
}

impl InterruptType {
    /// Get the bit position for this interrupt in IE/IF registers
    pub fn bit(&self) -> Byte {
        match self {
            InterruptType::VBlank => 0x01,
            InterruptType::LcdStat => 0x02,
            InterruptType::Timer => 0x04,
            InterruptType::Serial => 0x08,
            InterruptType::Joypad => 0x10,
        }
    }

    /// Get the interrupt vector address
    pub fn vector(&self) -> Word {
        match self {
            InterruptType::VBlank => 0x0040,
            InterruptType::LcdStat => 0x0048,
            InterruptType::Timer => 0x0050,
            InterruptType::Serial => 0x0058,
            InterruptType::Joypad => 0x0060,
        }
    }

    /// Get all interrupt types in priority order
    pub fn all() -> &'static [InterruptType] {
        &[
            InterruptType::VBlank,
            InterruptType::LcdStat,
            InterruptType::Timer,
            InterruptType::Serial,
            InterruptType::Joypad,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_new() {
        let cpu = Cpu::new();
        assert_eq!(cpu.regs.pc, 0);
        assert_eq!(cpu.regs.sp, 0);
        assert!(!cpu.halted);
        assert!(!cpu.ime);
    }

    #[test]
    fn test_cpu_init_boot_skip() {
        let mut cpu = Cpu::new();
        cpu.init();

        assert_eq!(cpu.regs.pc, 0x0100);
        assert_eq!(cpu.regs.sp, 0xFFFE);
        assert_eq!(cpu.regs.af(), 0x01B0);
        assert_eq!(cpu.regs.bc(), 0x0013);
        assert_eq!(cpu.regs.de(), 0x00D8);
        assert_eq!(cpu.regs.hl(), 0x014D);

        // Check flags: Z=1, N=0, H=1, C=1
        assert!(cpu.regs.flag_z());
        assert!(!cpu.regs.flag_n());
        assert!(cpu.regs.flag_h());
        assert!(cpu.regs.flag_c());
    }

    #[test]
    fn test_interrupt_request() {
        let mut cpu = Cpu::new();

        cpu.request_interrupt(InterruptType::VBlank);
        assert_eq!(cpu.int_flags, 0x01);

        cpu.request_interrupt(InterruptType::Timer);
        assert_eq!(cpu.int_flags, 0x05);
    }

    #[test]
    fn test_interrupt_pending() {
        let mut cpu = Cpu::new();

        // No interrupts pending initially
        assert!(!cpu.interrupts_pending());

        // Request interrupt but not enabled
        cpu.request_interrupt(InterruptType::VBlank);
        assert!(!cpu.interrupts_pending());

        // Enable interrupt
        cpu.ie_register = 0x01;
        assert!(cpu.interrupts_pending());
    }

    #[test]
    fn test_get_pending_interrupt_priority() {
        let mut cpu = Cpu::new();
        cpu.ie_register = 0x1F; // Enable all interrupts

        // Request Timer and Joypad
        cpu.request_interrupt(InterruptType::Timer);
        cpu.request_interrupt(InterruptType::Joypad);

        // Timer should be returned (higher priority)
        assert_eq!(cpu.get_pending_interrupt(), Some(InterruptType::Timer));

        // Request VBlank (highest priority)
        cpu.request_interrupt(InterruptType::VBlank);
        assert_eq!(cpu.get_pending_interrupt(), Some(InterruptType::VBlank));
    }

    #[test]
    fn test_interrupt_vectors() {
        assert_eq!(InterruptType::VBlank.vector(), 0x0040);
        assert_eq!(InterruptType::LcdStat.vector(), 0x0048);
        assert_eq!(InterruptType::Timer.vector(), 0x0050);
        assert_eq!(InterruptType::Serial.vector(), 0x0058);
        assert_eq!(InterruptType::Joypad.vector(), 0x0060);
    }

    #[test]
    fn test_clear_interrupt() {
        let mut cpu = Cpu::new();
        cpu.int_flags = 0x1F; // All flags set

        cpu.clear_interrupt(InterruptType::VBlank);
        assert_eq!(cpu.int_flags, 0x1E);

        cpu.clear_interrupt(InterruptType::Timer);
        assert_eq!(cpu.int_flags, 0x1A);
    }
}
