//! PPU Module
//!
//! This module implements the Pixel Processing Unit (PPU) for the Game Boy.
//! The PPU is responsible for rendering graphics to the screen.

pub mod modes;
pub mod pipeline;

use crate::common::{bit, Byte, Word};
use crate::lcd::{Lcd, PpuMode};

/// Screen dimensions
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;
pub const LINES_PER_FRAME: u8 = 154;
pub const TICKS_PER_LINE: u32 = 456;

/// OAM Entry (sprite attributes)
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct OamEntry {
    /// Y position (minus 16)
    pub y: Byte,
    /// X position (minus 8)
    pub x: Byte,
    /// Tile index
    pub tile: Byte,
    /// Flags (priority, flip, palette)
    pub flags: Byte,
}

impl OamEntry {
    /// CGB palette number (bits 0-2)
    pub fn cgb_palette(&self) -> Byte {
        self.flags & 0x07
    }

    /// CGB VRAM bank (bit 3)
    pub fn cgb_vram_bank(&self) -> bool {
        bit(self.flags, 3)
    }

    /// DMG palette number (bit 4)
    pub fn palette_number(&self) -> bool {
        bit(self.flags, 4)
    }

    /// X flip (bit 5)
    pub fn x_flip(&self) -> bool {
        bit(self.flags, 5)
    }

    /// Y flip (bit 6)
    pub fn y_flip(&self) -> bool {
        bit(self.flags, 6)
    }

    /// BG/Window over OBJ priority (bit 7)
    pub fn bg_priority(&self) -> bool {
        bit(self.flags, 7)
    }
}

/// Pixel Processing Unit
#[derive(Debug)]
pub struct Ppu {
    /// Video RAM (8KB)
    pub vram: [Byte; 0x2000],
    /// Object Attribute Memory (40 sprites * 4 bytes)
    pub oam: [Byte; 160],
    /// Video buffer (160x144 pixels, ARGB format)
    pub video_buffer: Vec<u32>,
    /// Current frame number
    pub current_frame: u32,
    /// Ticks within current line
    pub line_ticks: u32,
    /// Window internal line counter
    pub window_line: u8,
    /// VBlank interrupt requested
    pub vblank_interrupt: bool,
    /// Sprites on current line (max 10)
    pub line_sprites: Vec<OamEntry>,
    /// Number of sprites on current line
    pub sprite_count: usize,
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}

impl Ppu {
    /// Create a new PPU
    pub fn new() -> Self {
        Self {
            vram: [0; 0x2000],
            oam: [0; 160],
            video_buffer: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT],
            current_frame: 0,
            line_ticks: 0,
            window_line: 0,
            vblank_interrupt: false,
            line_sprites: Vec::with_capacity(10),
            sprite_count: 0,
        }
    }

    /// Initialize PPU
    pub fn init(&mut self) {
        self.vram.fill(0);
        self.oam.fill(0);
        self.video_buffer.fill(0);
        self.current_frame = 0;
        self.line_ticks = 0;
        self.window_line = 0;
        self.vblank_interrupt = false;
        self.line_sprites.clear();
        self.sprite_count = 0;
    }

    /// Read from VRAM
    pub fn vram_read(&self, address: Word) -> Byte {
        let offset = (address - 0x8000) as usize;
        if offset < self.vram.len() {
            self.vram[offset]
        } else {
            0xFF
        }
    }

    /// Write to VRAM
    pub fn vram_write(&mut self, address: Word, value: Byte) {
        let offset = (address - 0x8000) as usize;
        if offset < self.vram.len() {
            self.vram[offset] = value;
        }
    }

    /// Read from OAM
    pub fn oam_read(&self, address: Word) -> Byte {
        let offset = (address - 0xFE00) as usize;
        if offset < self.oam.len() {
            self.oam[offset]
        } else {
            0xFF
        }
    }

    /// Write to OAM
    pub fn oam_write(&mut self, address: Word, value: Byte) {
        let offset = (address - 0xFE00) as usize;
        if offset < self.oam.len() {
            self.oam[offset] = value;
        }
    }

    /// Get OAM entry at index
    pub fn get_oam_entry(&self, index: usize) -> OamEntry {
        if index >= 40 {
            return OamEntry::default();
        }
        let offset = index * 4;
        OamEntry {
            y: self.oam[offset],
            x: self.oam[offset + 1],
            tile: self.oam[offset + 2],
            flags: self.oam[offset + 3],
        }
    }

    /// Tick the PPU by one T-cycle
    pub fn tick(&mut self, lcd: &mut Lcd) {
        if !lcd.lcd_enabled() {
            return;
        }

        self.line_ticks += 1;

        match lcd.mode() {
            PpuMode::OamScan => self.mode_oam_scan(lcd),
            PpuMode::Transfer => self.mode_transfer(lcd),
            PpuMode::HBlank => self.mode_hblank(lcd),
            PpuMode::VBlank => self.mode_vblank(lcd),
        }
    }

    /// OAM Scan mode (mode 2) - 80 T-cycles
    fn mode_oam_scan(&mut self, lcd: &mut Lcd) {
        if self.line_ticks >= 80 {
            // Scan OAM for sprites on this line
            self.scan_oam(lcd);
            lcd.set_mode(PpuMode::Transfer);
        }
    }

    /// Pixel Transfer mode (mode 3) - variable length
    fn mode_transfer(&mut self, lcd: &mut Lcd) {
        // Simplified: assume fixed 172 T-cycles for transfer
        if self.line_ticks >= 80 + 172 {
            // Render the scanline
            self.render_scanline(lcd);
            lcd.set_mode(PpuMode::HBlank);
        }
    }

    /// HBlank mode (mode 0) - remainder of 456 T-cycles
    fn mode_hblank(&mut self, lcd: &mut Lcd) {
        if self.line_ticks >= TICKS_PER_LINE {
            self.line_ticks = 0;
            lcd.inc_ly();

            if lcd.ly >= SCREEN_HEIGHT as u8 {
                // Enter VBlank
                lcd.set_mode(PpuMode::VBlank);
                self.vblank_interrupt = true;
                self.current_frame += 1;
            } else {
                lcd.set_mode(PpuMode::OamScan);
            }
        }
    }

    /// VBlank mode (mode 1) - 10 scanlines
    fn mode_vblank(&mut self, lcd: &mut Lcd) {
        if self.line_ticks >= TICKS_PER_LINE {
            self.line_ticks = 0;
            lcd.inc_ly();

            if lcd.ly >= LINES_PER_FRAME {
                lcd.set_ly(0);
                lcd.set_mode(PpuMode::OamScan);
                self.window_line = 0;
            }
        }
    }

    /// Scan OAM for sprites on current scanline
    fn scan_oam(&mut self, lcd: &Lcd) {
        self.line_sprites.clear();
        self.sprite_count = 0;

        let ly = lcd.ly as i32;
        let sprite_height = lcd.sprite_height() as i32;

        for i in 0..40 {
            if self.sprite_count >= 10 {
                break;
            }

            let entry = self.get_oam_entry(i);
            let sprite_y = entry.y as i32 - 16;

            // Check if sprite is on this scanline
            if ly >= sprite_y && ly < sprite_y + sprite_height {
                self.line_sprites.push(entry);
                self.sprite_count += 1;
            }
        }

        // Sort sprites by X position (lower X = higher priority)
        // For same X, earlier OAM index has priority (already in order)
        self.line_sprites.sort_by(|a, b| a.x.cmp(&b.x));
    }

    /// Render a single scanline
    fn render_scanline(&mut self, lcd: &Lcd) {
        let ly = lcd.ly as usize;
        if ly >= SCREEN_HEIGHT {
            return;
        }

        for x in 0..SCREEN_WIDTH {
            let mut color = 0u8;

            // Render background
            if lcd.bg_window_enabled() {
                color = self.get_bg_pixel(lcd, x as u8, ly as u8);
            }

            // Render window
            if lcd.window_enabled() && lcd.bg_window_enabled() {
                if let Some(win_color) = self.get_window_pixel(lcd, x as u8, ly as u8) {
                    color = win_color;
                }
            }

            // Render sprites
            if lcd.sprites_enabled() {
                if let Some((sprite_color, priority)) = self.get_sprite_pixel(lcd, x as u8, ly as u8) {
                    // Sprite pixel is visible if:
                    // - BG priority is false, OR
                    // - BG color is 0 (transparent)
                    if !priority || color == 0 {
                        color = sprite_color;
                    }
                }
            }

            // Convert color to ARGB
            let argb = self.color_to_argb(color);
            self.video_buffer[ly * SCREEN_WIDTH + x] = argb;
        }

        // Increment window line counter if window was visible
        if lcd.window_enabled() && lcd.wy <= lcd.ly && lcd.wx <= 166 {
            self.window_line += 1;
        }
    }

    /// Get background pixel color at position
    fn get_bg_pixel(&self, lcd: &Lcd, x: u8, y: u8) -> u8 {
        let scroll_x = lcd.scx.wrapping_add(x);
        let scroll_y = lcd.scy.wrapping_add(y);

        let tile_map = lcd.bg_tile_map();
        let tile_data = lcd.bg_tile_data();

        self.get_tile_pixel(tile_map, tile_data, scroll_x, scroll_y, lcd)
    }

    /// Get window pixel color at position (if visible)
    fn get_window_pixel(&self, lcd: &Lcd, x: u8, y: u8) -> Option<u8> {
        // Window is visible if WX <= 166 and WY <= LY
        if lcd.wx > 166 || lcd.wy > y {
            return None;
        }

        let win_x = x as i16 - (lcd.wx as i16 - 7);
        if win_x < 0 {
            return None;
        }

        let tile_map = lcd.window_tile_map();
        let tile_data = lcd.bg_tile_data();

        Some(self.get_tile_pixel(tile_map, tile_data, win_x as u8, self.window_line, lcd))
    }

    /// Get tile pixel from tile map
    fn get_tile_pixel(&self, tile_map: u16, tile_data: u16, x: u8, y: u8, lcd: &Lcd) -> u8 {
        // Get tile coordinates
        let tile_x = (x / 8) as u16;
        let tile_y = (y / 8) as u16;

        // Get tile index from tile map
        let map_addr = tile_map + tile_y * 32 + tile_x;
        let tile_index = self.vram[(map_addr - 0x8000) as usize];

        // Get tile data address
        let tile_addr = if tile_data == 0x8000 {
            // Unsigned addressing
            tile_data + (tile_index as u16) * 16
        } else {
            // Signed addressing (0x8800 base, tile 0 at 0x9000)
            let signed_index = tile_index as i8 as i16;
            (0x9000i32 + (signed_index as i32) * 16) as u16
        };

        // Get pixel within tile
        let pixel_x = 7 - (x % 8);
        let pixel_y = (y % 8) * 2;

        let addr = (tile_addr - 0x8000 + pixel_y as u16) as usize;
        if addr + 1 >= self.vram.len() {
            return 0;
        }

        let lo = self.vram[addr];
        let hi = self.vram[addr + 1];

        let color_bit = ((hi >> pixel_x) & 1) << 1 | ((lo >> pixel_x) & 1);
        lcd.bg_color(color_bit)
    }

    /// Get sprite pixel at position (if any)
    fn get_sprite_pixel(&self, lcd: &Lcd, x: u8, y: u8) -> Option<(u8, bool)> {
        let sprite_height = lcd.sprite_height();

        for sprite in &self.line_sprites {
            let sprite_x = sprite.x as i16 - 8;
            let sprite_y = sprite.y as i16 - 16;

            // Check if pixel is within sprite bounds
            if (x as i16) < sprite_x || (x as i16) >= sprite_x + 8 {
                continue;
            }

            let mut pixel_x = (x as i16 - sprite_x) as u8;
            let mut pixel_y = (y as i16 - sprite_y) as u8;

            // Handle flipping
            if sprite.x_flip() {
                pixel_x = 7 - pixel_x;
            }
            if sprite.y_flip() {
                pixel_y = sprite_height - 1 - pixel_y;
            }

            // Get tile index (mask bit 0 for 8x16 sprites)
            let tile_index = if sprite_height == 16 {
                sprite.tile & 0xFE
            } else {
                sprite.tile
            };

            // Get tile data
            let tile_addr = 0x8000u16 + (tile_index as u16) * 16 + (pixel_y as u16) * 2;
            let addr = (tile_addr - 0x8000) as usize;
            
            if addr + 1 >= self.vram.len() {
                continue;
            }

            let lo = self.vram[addr];
            let hi = self.vram[addr + 1];

            let color_bit = ((hi >> (7 - pixel_x)) & 1) << 1 | ((lo >> (7 - pixel_x)) & 1);

            // Color 0 is transparent for sprites
            if color_bit == 0 {
                continue;
            }

            // Get color from appropriate palette
            let color = if sprite.palette_number() {
                lcd.sprite_color_1(color_bit)
            } else {
                lcd.sprite_color_0(color_bit)
            };

            return Some((color, sprite.bg_priority()));
        }

        None
    }

    /// Convert 2-bit color to ARGB
    fn color_to_argb(&self, color: u8) -> u32 {
        // Classic Game Boy green palette
        match color & 0x03 {
            0 => 0xFF9BBC0F, // Lightest
            1 => 0xFF8BAC0F,
            2 => 0xFF306230,
            3 => 0xFF0F380F, // Darkest
            _ => 0xFF000000,
        }
    }

    /// Clear VBlank interrupt flag
    pub fn clear_vblank_interrupt(&mut self) {
        self.vblank_interrupt = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppu_new() {
        let ppu = Ppu::new();
        assert_eq!(ppu.vram.len(), 0x2000);
        assert_eq!(ppu.oam.len(), 160);
        assert_eq!(ppu.video_buffer.len(), SCREEN_WIDTH * SCREEN_HEIGHT);
    }

    #[test]
    fn test_vram_read_write() {
        let mut ppu = Ppu::new();
        
        ppu.vram_write(0x8000, 0x42);
        assert_eq!(ppu.vram_read(0x8000), 0x42);
        
        ppu.vram_write(0x9FFF, 0x55);
        assert_eq!(ppu.vram_read(0x9FFF), 0x55);
    }

    #[test]
    fn test_oam_read_write() {
        let mut ppu = Ppu::new();
        
        ppu.oam_write(0xFE00, 0x10);
        assert_eq!(ppu.oam_read(0xFE00), 0x10);
        
        ppu.oam_write(0xFE9F, 0x20);
        assert_eq!(ppu.oam_read(0xFE9F), 0x20);
    }

    #[test]
    fn test_oam_entry() {
        let mut ppu = Ppu::new();
        
        // Set up sprite 0
        ppu.oam[0] = 32;  // Y
        ppu.oam[1] = 16;  // X
        ppu.oam[2] = 5;   // Tile
        ppu.oam[3] = 0b11110000; // Flags
        
        let entry = ppu.get_oam_entry(0);
        assert_eq!(entry.y, 32);
        assert_eq!(entry.x, 16);
        assert_eq!(entry.tile, 5);
        assert!(entry.bg_priority());
        assert!(entry.y_flip());
        assert!(entry.x_flip());
        assert!(entry.palette_number());
    }

    #[test]
    fn test_color_to_argb() {
        let ppu = Ppu::new();
        
        assert_eq!(ppu.color_to_argb(0), 0xFF9BBC0F);
        assert_eq!(ppu.color_to_argb(3), 0xFF0F380F);
    }
}
