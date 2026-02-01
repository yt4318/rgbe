//! Memory Bus
//!
//! This module implements the Game Boy memory bus, which routes
//! memory accesses to the appropriate hardware components based on address.

use crate::common::{Byte, Word};

/// Memory bus trait for reading and writing memory
pub trait MemoryBus {
    /// Read a byte from the given address
    fn read(&self, address: Word) -> Byte;
    
    /// Write a byte to the given address
    fn write(&mut self, address: Word, value: Byte);
    
    /// Read a 16-bit word from the given address (little-endian)
    fn read16(&self, address: Word) -> Word {
        let lo = self.read(address) as Word;
        let hi = self.read(address.wrapping_add(1)) as Word;
        lo | (hi << 8)
    }
    
    /// Write a 16-bit word to the given address (little-endian)
    fn write16(&mut self, address: Word, value: Word) {
        self.write(address, (value & 0xFF) as Byte);
        self.write(address.wrapping_add(1), ((value >> 8) & 0xFF) as Byte);
    }
}

use crate::cart::Cartridge;
use crate::ram::Ram;

/// Game Boy memory bus
/// 
/// Routes memory accesses to the appropriate hardware components:
/// - 0x0000-0x7FFF: Cartridge ROM
/// - 0x8000-0x9FFF: PPU VRAM
/// - 0xA000-0xBFFF: Cartridge RAM
/// - 0xC000-0xDFFF: WRAM
/// - 0xE000-0xFDFF: Echo RAM (returns 0)
/// - 0xFE00-0xFE9F: PPU OAM
/// - 0xFEA0-0xFEFF: Unusable (returns 0)
/// - 0xFF00-0xFF7F: I/O registers
/// - 0xFF80-0xFFFE: HRAM
/// - 0xFFFF: IE register
pub struct Bus {
    /// RAM (WRAM + HRAM)
    pub ram: Ram,
    /// IE register (stored in CPU, but accessed via bus at 0xFFFF)
    pub ie_register: Byte,
    /// Interrupt flags register (0xFF0F)
    pub int_flags: Byte,
    /// Cartridge (handles MBC)
    pub cart: Option<Cartridge>,
    /// VRAM (shared with PPU)
    pub vram: [Byte; 0x2000],
    /// OAM (shared with PPU)
    pub oam: [Byte; 0xA0],
    /// I/O registers
    pub io_regs: [Byte; 0x80],
    /// DMA transferring flag
    pub dma_active: bool,
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

impl Bus {
    /// Create a new bus with all memory zeroed
    pub fn new() -> Self {
        Self {
            ram: Ram::new(),
            ie_register: 0,
            int_flags: 0,
            cart: None,
            vram: [0; 0x2000],
            oam: [0; 0xA0],
            io_regs: [0; 0x80],
            dma_active: false,
        }
    }

    /// Load cartridge into bus
    pub fn load_cartridge(&mut self, cart: Cartridge) {
        self.cart = Some(cart);
    }

    /// Set DMA active state
    pub fn set_dma_active(&mut self, active: bool) {
        self.dma_active = active;
    }

    /// Check if DMA is active
    pub fn is_dma_active(&self) -> bool {
        self.dma_active
    }

    /// Save cartridge battery (if applicable)
    pub fn save_battery(&mut self) {
        if let Some(ref mut cart) = self.cart {
            let _ = cart.save_battery();
        }
    }
}

impl MemoryBus for Bus {
    fn read(&self, address: Word) -> Byte {
        match address {
            // Cartridge ROM (0x0000-0x7FFF)
            0x0000..=0x7FFF => {
                if let Some(ref cart) = self.cart {
                    cart.read(address)
                } else {
                    0xFF
                }
            }
            // VRAM (0x8000-0x9FFF)
            0x8000..=0x9FFF => {
                self.vram[(address - 0x8000) as usize]
            }
            // Cartridge RAM (0xA000-0xBFFF)
            0xA000..=0xBFFF => {
                if let Some(ref cart) = self.cart {
                    cart.read(address)
                } else {
                    0xFF
                }
            }
            // WRAM (0xC000-0xDFFF)
            0xC000..=0xDFFF => {
                self.ram.wram_read(address)
            }
            // Echo RAM (0xE000-0xFDFF) - mirror of WRAM
            0xE000..=0xFDFF => {
                self.ram.wram_read(address - 0x2000)
            }
            // OAM (0xFE00-0xFE9F)
            0xFE00..=0xFE9F => {
                if self.dma_active {
                    0xFF
                } else {
                    self.oam[(address - 0xFE00) as usize]
                }
            }
            // Unusable (0xFEA0-0xFEFF)
            0xFEA0..=0xFEFF => 0xFF,
            // I/O registers (0xFF00-0xFF7F)
            0xFF00..=0xFF7F => {
                // Special case for IF register
                if address == 0xFF0F {
                    self.int_flags | 0xE0
                } else {
                    self.io_regs[(address - 0xFF00) as usize]
                }
            }
            // HRAM (0xFF80-0xFFFE)
            0xFF80..=0xFFFE => {
                self.ram.hram_read(address)
            }
            // IE register (0xFFFF)
            0xFFFF => self.ie_register,
        }
    }

    fn write(&mut self, address: Word, value: Byte) {
        match address {
            // Cartridge ROM (0x0000-0x7FFF) - writes go to MBC
            0x0000..=0x7FFF => {
                if let Some(ref mut cart) = self.cart {
                    cart.write(address, value);
                }
            }
            // VRAM (0x8000-0x9FFF)
            0x8000..=0x9FFF => {
                self.vram[(address - 0x8000) as usize] = value;
            }
            // Cartridge RAM (0xA000-0xBFFF)
            0xA000..=0xBFFF => {
                if let Some(ref mut cart) = self.cart {
                    cart.write(address, value);
                }
            }
            // WRAM (0xC000-0xDFFF)
            0xC000..=0xDFFF => {
                self.ram.wram_write(address, value);
            }
            // Echo RAM (0xE000-0xFDFF) - mirror of WRAM
            0xE000..=0xFDFF => {
                self.ram.wram_write(address - 0x2000, value);
            }
            // OAM (0xFE00-0xFE9F)
            0xFE00..=0xFE9F => {
                if !self.dma_active {
                    self.oam[(address - 0xFE00) as usize] = value;
                }
            }
            // Unusable (0xFEA0-0xFEFF) - ignored
            0xFEA0..=0xFEFF => {}
            // I/O registers (0xFF00-0xFF7F)
            0xFF00..=0xFF7F => {
                // Special case for IF register
                if address == 0xFF0F {
                    self.int_flags = value;
                } else {
                    self.io_regs[(address - 0xFF00) as usize] = value;
                }
            }
            // HRAM (0xFF80-0xFFFE)
            0xFF80..=0xFFFE => {
                self.ram.hram_write(address, value);
            }
            // IE register (0xFFFF)
            0xFFFF => {
                self.ie_register = value;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wram_routing() {
        let mut bus = Bus::new();
        
        bus.write(0xC000, 0x42);
        assert_eq!(bus.read(0xC000), 0x42);
        
        bus.write(0xDFFF, 0xAB);
        assert_eq!(bus.read(0xDFFF), 0xAB);
    }

    #[test]
    fn test_hram_routing() {
        let mut bus = Bus::new();
        
        bus.write(0xFF80, 0x12);
        assert_eq!(bus.read(0xFF80), 0x12);
        
        bus.write(0xFFFE, 0x34);
        assert_eq!(bus.read(0xFFFE), 0x34);
    }

    #[test]
    fn test_ie_register() {
        let mut bus = Bus::new();
        
        bus.write(0xFFFF, 0x1F);
        assert_eq!(bus.read(0xFFFF), 0x1F);
        assert_eq!(bus.ie_register, 0x1F);
    }

    #[test]
    fn test_if_register() {
        let mut bus = Bus::new();
        
        bus.write(0xFF0F, 0x05);
        assert_eq!(bus.read(0xFF0F) & 0x1F, 0x05);
        assert_eq!(bus.int_flags, 0x05);
    }

    #[test]
    fn test_vram_routing() {
        let mut bus = Bus::new();
        
        bus.write(0x8000, 0x55);
        assert_eq!(bus.read(0x8000), 0x55);
        
        bus.write(0x9FFF, 0xAA);
        assert_eq!(bus.read(0x9FFF), 0xAA);
    }

    #[test]
    fn test_oam_routing() {
        let mut bus = Bus::new();
        
        bus.write(0xFE00, 0x11);
        assert_eq!(bus.read(0xFE00), 0x11);
        
        // Test DMA blocking
        bus.set_dma_active(true);
        assert_eq!(bus.read(0xFE00), 0xFF);
        bus.write(0xFE00, 0x22);
        bus.set_dma_active(false);
        assert_eq!(bus.read(0xFE00), 0x11); // Should not have changed
    }

    #[test]
    fn test_echo_ram() {
        let mut bus = Bus::new();
        // Echo RAM mirrors WRAM
        bus.write(0xC000, 0x42);
        assert_eq!(bus.read(0xE000), 0x42);
    }

    #[test]
    fn test_unusable_area() {
        let bus = Bus::new();
        assert_eq!(bus.read(0xFEA0), 0xFF);
        assert_eq!(bus.read(0xFEFF), 0xFF);
    }

    #[test]
    fn test_read16_write16() {
        let mut bus = Bus::new();
        
        bus.write16(0xC000, 0x1234);
        assert_eq!(bus.read(0xC000), 0x34); // Low byte
        assert_eq!(bus.read(0xC001), 0x12); // High byte
        assert_eq!(bus.read16(0xC000), 0x1234);
    }
}
