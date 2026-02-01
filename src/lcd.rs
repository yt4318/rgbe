//! LCD Control
//!
//! This module implements the LCD control registers for the Game Boy.
//!
//! LCD Registers:
//! - LCDC (0xFF40): LCD Control
//! - STAT (0xFF41): LCD Status
//! - SCY (0xFF42): Scroll Y
//! - SCX (0xFF43): Scroll X
//! - LY (0xFF44): Current scanline (read-only)
//! - LYC (0xFF45): LY Compare
//! - DMA (0xFF46): DMA Transfer (handled in dma.rs)
//! - BGP (0xFF47): Background Palette
//! - OBP0 (0xFF48): Object Palette 0
//! - OBP1 (0xFF49): Object Palette 1
//! - WY (0xFF4A): Window Y Position
//! - WX (0xFF4B): Window X Position

use crate::common::{bit, bit_set, Byte};

/// PPU modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PpuMode {
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Transfer = 3,
}

impl From<u8> for PpuMode {
    fn from(value: u8) -> Self {
        match value & 0x03 {
            0 => PpuMode::HBlank,
            1 => PpuMode::VBlank,
            2 => PpuMode::OamScan,
            3 => PpuMode::Transfer,
            _ => unreachable!(),
        }
    }
}

/// LCD Controller
#[derive(Debug, Clone)]
pub struct Lcd {
    /// LCDC - LCD Control (0xFF40)
    pub lcdc: Byte,
    /// STAT - LCD Status (0xFF41)
    pub stat: Byte,
    /// SCY - Scroll Y (0xFF42)
    pub scy: Byte,
    /// SCX - Scroll X (0xFF43)
    pub scx: Byte,
    /// LY - Current scanline (0xFF44)
    pub ly: Byte,
    /// LYC - LY Compare (0xFF45)
    pub lyc: Byte,
    /// BGP - Background Palette (0xFF47)
    pub bgp: Byte,
    /// OBP0 - Object Palette 0 (0xFF48)
    pub obp0: Byte,
    /// OBP1 - Object Palette 1 (0xFF49)
    pub obp1: Byte,
    /// WY - Window Y Position (0xFF4A)
    pub wy: Byte,
    /// WX - Window X Position (0xFF4B)
    pub wx: Byte,
    /// STAT interrupt requested
    pub stat_interrupt: bool,
}

impl Default for Lcd {
    fn default() -> Self {
        Self::new()
    }
}

impl Lcd {
    /// Create a new LCD with default state
    pub fn new() -> Self {
        Self {
            lcdc: 0x91, // LCD enabled, BG enabled
            stat: 0x02, // Start in OAM scan mode (mode 2)
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0xFC,
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0,
            wx: 0,
            stat_interrupt: false,
        }
    }

    /// Initialize LCD to boot ROM skip state
    pub fn init(&mut self) {
        self.lcdc = 0x91;
        self.stat = 0x02; // Start in OAM scan mode (mode 2)
        self.scy = 0;
        self.scx = 0;
        self.ly = 0;
        self.lyc = 0;
        self.bgp = 0xFC;
        self.obp0 = 0xFF;
        self.obp1 = 0xFF;
        self.wy = 0;
        self.wx = 0;
        self.stat_interrupt = false;
    }

    /// Read LCD register
    pub fn read(&self, address: u16) -> Byte {
        match address {
            0xFF40 => self.lcdc,
            0xFF41 => self.stat | 0x80, // Bit 7 always reads as 1
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => 0xFF,
        }
    }

    /// Write LCD register
    pub fn write(&mut self, address: u16, value: Byte) {
        match address {
            0xFF40 => self.lcdc = value,
            0xFF41 => {
                // Lower 3 bits are read-only (mode and LYC flag)
                self.stat = (self.stat & 0x07) | (value & 0xF8);
            }
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => {} // LY is read-only
            0xFF45 => {
                self.lyc = value;
                self.check_lyc();
            }
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,
            _ => {}
        }
    }

    // ========== LCDC Bit Accessors ==========

    /// LCD Display Enable (bit 7)
    pub fn lcd_enabled(&self) -> bool {
        bit(self.lcdc, 7)
    }

    /// Window Tile Map Select (bit 6)
    /// false = 0x9800-0x9BFF, true = 0x9C00-0x9FFF
    pub fn window_tile_map(&self) -> u16 {
        if bit(self.lcdc, 6) { 0x9C00 } else { 0x9800 }
    }

    /// Window Enable (bit 5)
    pub fn window_enabled(&self) -> bool {
        bit(self.lcdc, 5)
    }

    /// BG & Window Tile Data Select (bit 4)
    /// false = 0x8800-0x97FF (signed), true = 0x8000-0x8FFF (unsigned)
    pub fn bg_tile_data(&self) -> u16 {
        if bit(self.lcdc, 4) { 0x8000 } else { 0x8800 }
    }

    /// BG Tile Map Select (bit 3)
    /// false = 0x9800-0x9BFF, true = 0x9C00-0x9FFF
    pub fn bg_tile_map(&self) -> u16 {
        if bit(self.lcdc, 3) { 0x9C00 } else { 0x9800 }
    }

    /// Sprite Size (bit 2)
    /// false = 8x8, true = 8x16
    pub fn sprite_height(&self) -> u8 {
        if bit(self.lcdc, 2) { 16 } else { 8 }
    }

    /// Sprite Enable (bit 1)
    pub fn sprites_enabled(&self) -> bool {
        bit(self.lcdc, 1)
    }

    /// BG & Window Enable (bit 0)
    pub fn bg_window_enabled(&self) -> bool {
        bit(self.lcdc, 0)
    }

    // ========== STAT Bit Accessors ==========

    /// Get current PPU mode (bits 0-1)
    pub fn mode(&self) -> PpuMode {
        PpuMode::from(self.stat & 0x03)
    }

    /// Set current PPU mode (bits 0-1)
    pub fn set_mode(&mut self, mode: PpuMode) {
        self.stat = (self.stat & 0xFC) | (mode as u8);
        self.check_stat_interrupt();
    }

    /// LYC=LY Coincidence Flag (bit 2)
    pub fn lyc_flag(&self) -> bool {
        bit(self.stat, 2)
    }

    /// Set LYC=LY Coincidence Flag (bit 2)
    fn set_lyc_flag(&mut self, value: bool) {
        bit_set(&mut self.stat, 2, value);
    }

    /// Mode 0 HBlank Interrupt Enable (bit 3)
    pub fn hblank_int_enabled(&self) -> bool {
        bit(self.stat, 3)
    }

    /// Mode 1 VBlank Interrupt Enable (bit 4)
    pub fn vblank_int_enabled(&self) -> bool {
        bit(self.stat, 4)
    }

    /// Mode 2 OAM Interrupt Enable (bit 5)
    pub fn oam_int_enabled(&self) -> bool {
        bit(self.stat, 5)
    }

    /// LYC=LY Coincidence Interrupt Enable (bit 6)
    pub fn lyc_int_enabled(&self) -> bool {
        bit(self.stat, 6)
    }

    // ========== LY/LYC Handling ==========

    /// Set current scanline (LY)
    pub fn set_ly(&mut self, value: Byte) {
        self.ly = value;
        self.check_lyc();
    }

    /// Increment LY and check for LYC match
    pub fn inc_ly(&mut self) {
        self.ly = self.ly.wrapping_add(1);
        if self.ly > 153 {
            self.ly = 0;
        }
        self.check_lyc();
    }

    /// Check LY=LYC coincidence and request interrupt if enabled
    fn check_lyc(&mut self) {
        let coincidence = self.ly == self.lyc;
        self.set_lyc_flag(coincidence);
        
        if coincidence && self.lyc_int_enabled() {
            self.stat_interrupt = true;
        }
    }

    /// Check if STAT interrupt should be requested based on current mode
    fn check_stat_interrupt(&mut self) {
        let should_interrupt = match self.mode() {
            PpuMode::HBlank => self.hblank_int_enabled(),
            PpuMode::VBlank => self.vblank_int_enabled(),
            PpuMode::OamScan => self.oam_int_enabled(),
            PpuMode::Transfer => false,
        };
        
        if should_interrupt {
            self.stat_interrupt = true;
        }
    }

    /// Clear STAT interrupt flag
    pub fn clear_stat_interrupt(&mut self) {
        self.stat_interrupt = false;
    }

    // ========== Palette Helpers ==========

    /// Get color from background palette
    pub fn bg_color(&self, color_id: u8) -> u8 {
        (self.bgp >> (color_id * 2)) & 0x03
    }

    /// Get color from sprite palette 0
    pub fn sprite_color_0(&self, color_id: u8) -> u8 {
        (self.obp0 >> (color_id * 2)) & 0x03
    }

    /// Get color from sprite palette 1
    pub fn sprite_color_1(&self, color_id: u8) -> u8 {
        (self.obp1 >> (color_id * 2)) & 0x03
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lcd_new() {
        let lcd = Lcd::new();
        assert_eq!(lcd.lcdc, 0x91);
        assert_eq!(lcd.bgp, 0xFC);
        assert!(lcd.lcd_enabled());
        assert!(lcd.bg_window_enabled());
    }

    #[test]
    fn test_lcdc_bits() {
        let mut lcd = Lcd::new();
        
        lcd.lcdc = 0xFF;
        assert!(lcd.lcd_enabled());
        assert!(lcd.window_enabled());
        assert!(lcd.sprites_enabled());
        assert!(lcd.bg_window_enabled());
        assert_eq!(lcd.sprite_height(), 16);
        assert_eq!(lcd.window_tile_map(), 0x9C00);
        assert_eq!(lcd.bg_tile_map(), 0x9C00);
        assert_eq!(lcd.bg_tile_data(), 0x8000);
        
        lcd.lcdc = 0x00;
        assert!(!lcd.lcd_enabled());
        assert!(!lcd.window_enabled());
        assert!(!lcd.sprites_enabled());
        assert!(!lcd.bg_window_enabled());
        assert_eq!(lcd.sprite_height(), 8);
        assert_eq!(lcd.window_tile_map(), 0x9800);
        assert_eq!(lcd.bg_tile_map(), 0x9800);
        assert_eq!(lcd.bg_tile_data(), 0x8800);
    }

    #[test]
    fn test_stat_mode() {
        let mut lcd = Lcd::new();
        
        lcd.set_mode(PpuMode::OamScan);
        assert_eq!(lcd.mode(), PpuMode::OamScan);
        
        lcd.set_mode(PpuMode::Transfer);
        assert_eq!(lcd.mode(), PpuMode::Transfer);
        
        lcd.set_mode(PpuMode::HBlank);
        assert_eq!(lcd.mode(), PpuMode::HBlank);
        
        lcd.set_mode(PpuMode::VBlank);
        assert_eq!(lcd.mode(), PpuMode::VBlank);
    }

    #[test]
    fn test_lyc_coincidence() {
        let mut lcd = Lcd::new();
        lcd.stat = 0x40; // Enable LYC interrupt
        
        lcd.lyc = 10;
        lcd.set_ly(10);
        
        assert!(lcd.lyc_flag());
        assert!(lcd.stat_interrupt);
    }

    #[test]
    fn test_ly_read_only() {
        let mut lcd = Lcd::new();
        lcd.ly = 50;
        
        lcd.write(0xFF44, 0x00); // Try to write to LY
        
        assert_eq!(lcd.ly, 50); // Should be unchanged
    }

    #[test]
    fn test_palette_colors() {
        let mut lcd = Lcd::new();
        lcd.bgp = 0b11_10_01_00; // Colors: 3, 2, 1, 0
        
        assert_eq!(lcd.bg_color(0), 0);
        assert_eq!(lcd.bg_color(1), 1);
        assert_eq!(lcd.bg_color(2), 2);
        assert_eq!(lcd.bg_color(3), 3);
    }
}
