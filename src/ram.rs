//! RAM
//!
//! This module implements Work RAM (WRAM) and High RAM (HRAM) for the Game Boy.

use crate::common::{Byte, Word};

/// WRAM size: 8KB (0xC000-0xDFFF)
const WRAM_SIZE: usize = 0x2000;

/// HRAM size: 127 bytes (0xFF80-0xFFFE)
const HRAM_SIZE: usize = 0x7F;

/// RAM structure containing WRAM and HRAM
#[derive(Debug)]
pub struct Ram {
    /// Work RAM (8KB)
    wram: [Byte; WRAM_SIZE],
    /// High RAM (127 bytes)
    hram: [Byte; HRAM_SIZE],
}

impl Default for Ram {
    fn default() -> Self {
        Self::new()
    }
}

impl Ram {
    /// Create a new RAM instance with all memory zeroed
    pub fn new() -> Self {
        Self {
            wram: [0; WRAM_SIZE],
            hram: [0; HRAM_SIZE],
        }
    }

    /// Read from WRAM (0xC000-0xDFFF)
    pub fn wram_read(&self, address: Word) -> Byte {
        let offset = (address.wrapping_sub(0xC000)) as usize;
        if offset >= WRAM_SIZE {
            // Invalid address, return 0xFF
            return 0xFF;
        }
        self.wram[offset]
    }

    /// Write to WRAM (0xC000-0xDFFF)
    pub fn wram_write(&mut self, address: Word, value: Byte) {
        let offset = (address.wrapping_sub(0xC000)) as usize;
        if offset < WRAM_SIZE {
            self.wram[offset] = value;
        }
    }

    /// Read from HRAM (0xFF80-0xFFFE)
    pub fn hram_read(&self, address: Word) -> Byte {
        let offset = (address.wrapping_sub(0xFF80)) as usize;
        if offset >= HRAM_SIZE {
            // Invalid address, return 0xFF
            return 0xFF;
        }
        self.hram[offset]
    }

    /// Write to HRAM (0xFF80-0xFFFE)
    pub fn hram_write(&mut self, address: Word, value: Byte) {
        let offset = (address.wrapping_sub(0xFF80)) as usize;
        if offset < HRAM_SIZE {
            self.hram[offset] = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wram_read_write() {
        let mut ram = Ram::new();
        
        // Write and read at start of WRAM
        ram.wram_write(0xC000, 0x42);
        assert_eq!(ram.wram_read(0xC000), 0x42);
        
        // Write and read at end of WRAM
        ram.wram_write(0xDFFF, 0xAB);
        assert_eq!(ram.wram_read(0xDFFF), 0xAB);
        
        // Write and read in middle
        ram.wram_write(0xC100, 0x55);
        assert_eq!(ram.wram_read(0xC100), 0x55);
    }

    #[test]
    fn test_hram_read_write() {
        let mut ram = Ram::new();
        
        // Write and read at start of HRAM
        ram.hram_write(0xFF80, 0x12);
        assert_eq!(ram.hram_read(0xFF80), 0x12);
        
        // Write and read at end of HRAM
        ram.hram_write(0xFFFE, 0x34);
        assert_eq!(ram.hram_read(0xFFFE), 0x34);
        
        // Write and read in middle
        ram.hram_write(0xFFA0, 0x78);
        assert_eq!(ram.hram_read(0xFFA0), 0x78);
    }

    #[test]
    fn test_ram_initial_state() {
        let ram = Ram::new();
        
        // All memory should be zeroed initially
        assert_eq!(ram.wram_read(0xC000), 0);
        assert_eq!(ram.wram_read(0xDFFF), 0);
        assert_eq!(ram.hram_read(0xFF80), 0);
        assert_eq!(ram.hram_read(0xFFFE), 0);
    }
}
