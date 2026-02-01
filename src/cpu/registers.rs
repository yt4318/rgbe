//! CPU Registers
//!
//! This module defines the CPU register structure and accessors for the
//! Sharp LR35902 processor used in the Game Boy.

use crate::common::{bit, bit_set, Byte, Word};

/// CPU Registers
///
/// The Game Boy CPU has 8 8-bit registers (A, F, B, C, D, E, H, L)
/// and 2 16-bit registers (SP, PC). The 8-bit registers can be
/// combined into 16-bit register pairs (AF, BC, DE, HL).
#[derive(Debug, Clone, Copy, Default)]
pub struct Registers {
    /// Accumulator register
    pub a: Byte,
    /// Flags register (Z, N, H, C in bits 7, 6, 5, 4)
    pub f: Byte,
    /// General purpose register B
    pub b: Byte,
    /// General purpose register C
    pub c: Byte,
    /// General purpose register D
    pub d: Byte,
    /// General purpose register E
    pub e: Byte,
    /// General purpose register H
    pub h: Byte,
    /// General purpose register L
    pub l: Byte,
    /// Program Counter
    pub pc: Word,
    /// Stack Pointer
    pub sp: Word,
}

impl Registers {
    /// Create new registers with default values (all zeros)
    pub fn new() -> Self {
        Self::default()
    }

    // ========== 16-bit Register Pair Accessors ==========

    /// Get AF register pair (Accumulator + Flags)
    #[inline]
    pub fn af(&self) -> Word {
        ((self.a as Word) << 8) | (self.f as Word)
    }

    /// Set AF register pair
    /// Note: Lower 4 bits of F are always 0
    #[inline]
    pub fn set_af(&mut self, value: Word) {
        self.a = ((value >> 8) & 0xFF) as Byte;
        self.f = (value & 0xF0) as Byte; // Lower 4 bits are always 0
    }

    /// Get BC register pair
    #[inline]
    pub fn bc(&self) -> Word {
        ((self.b as Word) << 8) | (self.c as Word)
    }

    /// Set BC register pair
    #[inline]
    pub fn set_bc(&mut self, value: Word) {
        self.b = ((value >> 8) & 0xFF) as Byte;
        self.c = (value & 0xFF) as Byte;
    }

    /// Get DE register pair
    #[inline]
    pub fn de(&self) -> Word {
        ((self.d as Word) << 8) | (self.e as Word)
    }

    /// Set DE register pair
    #[inline]
    pub fn set_de(&mut self, value: Word) {
        self.d = ((value >> 8) & 0xFF) as Byte;
        self.e = (value & 0xFF) as Byte;
    }

    /// Get HL register pair
    #[inline]
    pub fn hl(&self) -> Word {
        ((self.h as Word) << 8) | (self.l as Word)
    }

    /// Set HL register pair
    #[inline]
    pub fn set_hl(&mut self, value: Word) {
        self.h = ((value >> 8) & 0xFF) as Byte;
        self.l = (value & 0xFF) as Byte;
    }

    // ========== Flag Accessors ==========
    // Flags are stored in the F register:
    // Bit 7: Z (Zero flag)
    // Bit 6: N (Subtract flag)
    // Bit 5: H (Half Carry flag)
    // Bit 4: C (Carry flag)
    // Bits 0-3: Always 0

    /// Get Zero flag (bit 7)
    #[inline]
    pub fn flag_z(&self) -> bool {
        bit(self.f, 7)
    }

    /// Set Zero flag (bit 7)
    #[inline]
    pub fn set_flag_z(&mut self, value: bool) {
        bit_set(&mut self.f, 7, value);
    }

    /// Get Subtract flag (bit 6)
    #[inline]
    pub fn flag_n(&self) -> bool {
        bit(self.f, 6)
    }

    /// Set Subtract flag (bit 6)
    #[inline]
    pub fn set_flag_n(&mut self, value: bool) {
        bit_set(&mut self.f, 6, value);
    }

    /// Get Half Carry flag (bit 5)
    #[inline]
    pub fn flag_h(&self) -> bool {
        bit(self.f, 5)
    }

    /// Set Half Carry flag (bit 5)
    #[inline]
    pub fn set_flag_h(&mut self, value: bool) {
        bit_set(&mut self.f, 5, value);
    }

    /// Get Carry flag (bit 4)
    #[inline]
    pub fn flag_c(&self) -> bool {
        bit(self.f, 4)
    }

    /// Set Carry flag (bit 4)
    #[inline]
    pub fn set_flag_c(&mut self, value: bool) {
        bit_set(&mut self.f, 4, value);
    }

    /// Set all flags at once
    #[inline]
    pub fn set_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        self.set_flag_z(z);
        self.set_flag_n(n);
        self.set_flag_h(h);
        self.set_flag_c(c);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_registers() {
        let regs = Registers::new();
        assert_eq!(regs.a, 0);
        assert_eq!(regs.f, 0);
        assert_eq!(regs.b, 0);
        assert_eq!(regs.c, 0);
        assert_eq!(regs.d, 0);
        assert_eq!(regs.e, 0);
        assert_eq!(regs.h, 0);
        assert_eq!(regs.l, 0);
        assert_eq!(regs.pc, 0);
        assert_eq!(regs.sp, 0);
    }

    #[test]
    fn test_af_register_pair() {
        let mut regs = Registers::new();
        
        regs.set_af(0x01B0);
        assert_eq!(regs.a, 0x01);
        assert_eq!(regs.f, 0xB0);
        assert_eq!(regs.af(), 0x01B0);
        
        // Lower 4 bits of F should always be 0
        regs.set_af(0xFFFF);
        assert_eq!(regs.a, 0xFF);
        assert_eq!(regs.f, 0xF0);
        assert_eq!(regs.af(), 0xFFF0);
    }

    #[test]
    fn test_bc_register_pair() {
        let mut regs = Registers::new();
        
        regs.set_bc(0x0013);
        assert_eq!(regs.b, 0x00);
        assert_eq!(regs.c, 0x13);
        assert_eq!(regs.bc(), 0x0013);
        
        regs.set_bc(0xABCD);
        assert_eq!(regs.b, 0xAB);
        assert_eq!(regs.c, 0xCD);
        assert_eq!(regs.bc(), 0xABCD);
    }

    #[test]
    fn test_de_register_pair() {
        let mut regs = Registers::new();
        
        regs.set_de(0x00D8);
        assert_eq!(regs.d, 0x00);
        assert_eq!(regs.e, 0xD8);
        assert_eq!(regs.de(), 0x00D8);
    }

    #[test]
    fn test_hl_register_pair() {
        let mut regs = Registers::new();
        
        regs.set_hl(0x014D);
        assert_eq!(regs.h, 0x01);
        assert_eq!(regs.l, 0x4D);
        assert_eq!(regs.hl(), 0x014D);
    }

    #[test]
    fn test_flags() {
        let mut regs = Registers::new();
        
        // Test individual flags
        regs.set_flag_z(true);
        assert!(regs.flag_z());
        assert_eq!(regs.f, 0x80);
        
        regs.set_flag_n(true);
        assert!(regs.flag_n());
        assert_eq!(regs.f, 0xC0);
        
        regs.set_flag_h(true);
        assert!(regs.flag_h());
        assert_eq!(regs.f, 0xE0);
        
        regs.set_flag_c(true);
        assert!(regs.flag_c());
        assert_eq!(regs.f, 0xF0);
        
        // Clear flags
        regs.set_flag_z(false);
        assert!(!regs.flag_z());
        assert_eq!(regs.f, 0x70);
    }

    #[test]
    fn test_set_flags() {
        let mut regs = Registers::new();
        
        regs.set_flags(true, false, true, false);
        assert!(regs.flag_z());
        assert!(!regs.flag_n());
        assert!(regs.flag_h());
        assert!(!regs.flag_c());
        assert_eq!(regs.f, 0xA0);
        
        regs.set_flags(false, true, false, true);
        assert!(!regs.flag_z());
        assert!(regs.flag_n());
        assert!(!regs.flag_h());
        assert!(regs.flag_c());
        assert_eq!(regs.f, 0x50);
    }
}
