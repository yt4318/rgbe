//! Cartridge
//!
//! This module handles Game Boy cartridge emulation, including
//! ROM header parsing, MBC (Memory Bank Controller) support, and battery backup.

use crate::common::{Byte, Word};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

/// ROM header offsets
const HEADER_TITLE_START: usize = 0x134;
const HEADER_TITLE_END: usize = 0x143;
const HEADER_CART_TYPE: usize = 0x147;
const HEADER_ROM_SIZE: usize = 0x148;
const HEADER_RAM_SIZE: usize = 0x149;
const HEADER_LIC_CODE: usize = 0x14B;
const HEADER_VERSION: usize = 0x14C;
const HEADER_CHECKSUM: usize = 0x14D;

/// ROM header information
#[derive(Debug, Clone)]
pub struct RomHeader {
    /// Game title (up to 16 characters)
    pub title: String,
    /// Cartridge type (MBC type)
    pub cart_type: Byte,
    /// ROM size code
    pub rom_size: Byte,
    /// RAM size code
    pub ram_size: Byte,
    /// License code
    pub lic_code: Byte,
    /// Version number
    pub version: Byte,
    /// Header checksum
    pub checksum: Byte,
}

impl RomHeader {
    /// Parse ROM header from ROM data
    pub fn parse(rom_data: &[Byte]) -> Option<Self> {
        if rom_data.len() < 0x150 {
            return None;
        }

        // Extract title (null-terminated string)
        let title_bytes = &rom_data[HEADER_TITLE_START..=HEADER_TITLE_END];
        let title = title_bytes
            .iter()
            .take_while(|&&b| b != 0)
            .map(|&b| b as char)
            .collect();

        Some(Self {
            title,
            cart_type: rom_data[HEADER_CART_TYPE],
            rom_size: rom_data[HEADER_ROM_SIZE],
            ram_size: rom_data[HEADER_RAM_SIZE],
            lic_code: rom_data[HEADER_LIC_CODE],
            version: rom_data[HEADER_VERSION],
            checksum: rom_data[HEADER_CHECKSUM],
        })
    }

    /// Get ROM size in bytes
    pub fn rom_size_bytes(&self) -> usize {
        32768 << self.rom_size as usize
    }

    /// Get RAM size in bytes
    pub fn ram_size_bytes(&self) -> usize {
        match self.ram_size {
            0 => 0,
            1 => 2048,    // 2KB (unused)
            2 => 8192,    // 8KB
            3 => 32768,   // 32KB (4 banks)
            4 => 131072,  // 128KB (16 banks)
            5 => 65536,   // 64KB (8 banks)
            _ => 0,
        }
    }

    /// Get cartridge type name
    pub fn cart_type_name(&self) -> &'static str {
        match self.cart_type {
            0x00 => "ROM ONLY",
            0x01 => "MBC1",
            0x02 => "MBC1+RAM",
            0x03 => "MBC1+RAM+BATTERY",
            0x05 => "MBC2",
            0x06 => "MBC2+BATTERY",
            0x08 => "ROM+RAM",
            0x09 => "ROM+RAM+BATTERY",
            0x0F => "MBC3+TIMER+BATTERY",
            0x10 => "MBC3+TIMER+RAM+BATTERY",
            0x11 => "MBC3",
            0x12 => "MBC3+RAM",
            0x13 => "MBC3+RAM+BATTERY",
            0x19 => "MBC5",
            0x1A => "MBC5+RAM",
            0x1B => "MBC5+RAM+BATTERY",
            _ => "UNKNOWN",
        }
    }

    /// Check if cartridge has battery backup
    pub fn has_battery(&self) -> bool {
        matches!(self.cart_type, 0x03 | 0x06 | 0x09 | 0x0F | 0x10 | 0x13 | 0x1B)
    }

    /// Check if cartridge has RAM
    pub fn has_ram(&self) -> bool {
        matches!(self.cart_type, 0x02 | 0x03 | 0x08 | 0x09 | 0x10 | 0x12 | 0x13 | 0x1A | 0x1B)
    }
}

/// Cartridge emulation
#[derive(Debug)]
pub struct Cartridge {
    /// ROM file path
    filename: String,
    /// ROM data
    pub rom: Vec<Byte>,
    /// Parsed ROM header
    pub header: RomHeader,
    /// RAM enabled flag (for MBC)
    ram_enabled: bool,
    /// Current ROM bank (1-based for bank 1+)
    rom_bank: u8,
    /// Current RAM bank
    ram_bank: u8,
    /// Banking mode (0 = ROM, 1 = RAM)
    banking_mode: u8,
    /// Cartridge RAM
    ram: Vec<Byte>,
    /// Battery backup flag
    battery: bool,
    /// RAM needs to be saved
    need_save: bool,
}

impl Cartridge {
    /// Number of 16KB ROM banks available in this cartridge
    fn rom_bank_count(&self) -> usize {
        (self.rom.len() / 0x4000).max(1)
    }

    /// Number of 8KB RAM banks available in this cartridge
    fn ram_bank_count(&self) -> usize {
        (self.ram.len() / 0x2000).max(1)
    }

    /// Resolve effective MBC1 bank for 0x0000-0x3FFF region
    fn mbc1_rom0_bank(&self) -> usize {
        if self.banking_mode == 1 {
            // In mode 1, high bank bits affect bank 0 region on larger ROMs.
            let bank = ((self.ram_bank as usize) & 0x03) << 5;
            bank % self.rom_bank_count()
        } else {
            0
        }
    }

    /// Resolve effective MBC1 bank for 0x4000-0x7FFF region
    fn mbc1_romx_bank(&self) -> usize {
        let mut bank = (self.rom_bank as usize) & 0x1F;
        if self.banking_mode == 0 {
            bank |= ((self.ram_bank as usize) & 0x03) << 5;
        }

        // MBC1 cannot select banks where low 5 bits are all zero.
        if (bank & 0x1F) == 0 {
            bank = bank.wrapping_add(1);
        }

        let bank_count = self.rom_bank_count();
        bank %= bank_count;

        // 0x4000-0x7FFF should never map bank 0.
        if bank == 0 && bank_count > 1 {
            1
        } else {
            bank
        }
    }

    /// Load a cartridge from a ROM file
    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();
        let filename = path.to_string_lossy().to_string();
        
        let rom = fs::read(path)?;
        
        let header = RomHeader::parse(&rom)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid ROM header"))?;
        
        // Validate checksum
        if !Self::validate_checksum(&rom) {
            eprintln!("Warning: ROM header checksum invalid");
        }
        
        let ram_size = header.ram_size_bytes();
        let battery = header.has_battery();
        
        let mut cart = Self {
            filename: filename.clone(),
            rom,
            header,
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            banking_mode: 0,
            ram: vec![0; ram_size],
            battery,
            need_save: false,
        };
        
        // Load battery save if exists
        if battery {
            cart.load_battery_save();
        }
        
        Ok(cart)
    }

    /// Validate ROM header checksum
    pub fn validate_checksum(rom_data: &[Byte]) -> bool {
        if rom_data.len() < 0x150 {
            return false;
        }
        
        let mut checksum: u8 = 0;
        for i in 0x134..=0x14C {
            checksum = checksum.wrapping_sub(rom_data[i]).wrapping_sub(1);
        }
        
        checksum == rom_data[HEADER_CHECKSUM]
    }

    /// Calculate header checksum
    pub fn calculate_checksum(rom_data: &[Byte]) -> Byte {
        if rom_data.len() < 0x14D {
            return 0;
        }
        
        let mut checksum: u8 = 0;
        for i in 0x134..=0x14C {
            checksum = checksum.wrapping_sub(rom_data[i]).wrapping_sub(1);
        }
        checksum
    }

    /// Read from cartridge
    pub fn read(&self, address: Word) -> Byte {
        match address {
            // ROM Bank 0 (0x0000-0x3FFF)
            0x0000..=0x3FFF => {
                if self.is_mbc1() {
                    let bank = self.mbc1_rom0_bank();
                    let addr = (bank * 0x4000) + (address as usize);
                    self.rom.get(addr).copied().unwrap_or(0xFF)
                } else {
                    self.rom.get(address as usize).copied().unwrap_or(0xFF)
                }
            }
            // ROM Bank 1-N (0x4000-0x7FFF)
            0x4000..=0x7FFF => {
                let bank = if self.is_mbc1() {
                    self.mbc1_romx_bank()
                } else {
                    let bank_count = self.rom_bank_count();
                    let mut bank = (self.rom_bank as usize) % bank_count;
                    if bank == 0 && bank_count > 1 {
                        bank = 1;
                    }
                    bank
                };
                
                let addr = (bank * 0x4000) + ((address as usize) - 0x4000);
                self.rom.get(addr).copied().unwrap_or(0xFF)
            }
            // Cartridge RAM (0xA000-0xBFFF)
            0xA000..=0xBFFF => {
                if !self.ram_enabled || self.ram.is_empty() {
                    return 0xFF;
                }
                
                // MBC3: RAM bank 0-3 (RTC registers 0x08-0x0C not implemented)
                // MBC1: RAM bank depends on banking_mode
                let bank = if self.is_mbc3() {
                    (self.ram_bank & 0x03) as usize
                } else if self.banking_mode == 1 {
                    self.ram_bank as usize
                } else {
                    0
                };
                let bank = bank % self.ram_bank_count();
                let addr = (bank * 0x2000) + ((address as usize) - 0xA000);
                self.ram.get(addr).copied().unwrap_or(0xFF)
            }
            _ => 0xFF,
        }
    }

    /// Write to cartridge (MBC registers or RAM)
    pub fn write(&mut self, address: Word, value: Byte) {
        match address {
            // RAM Enable (0x0000-0x1FFF)
            0x0000..=0x1FFF => {
                if self.is_mbc1() || self.is_mbc3() || self.is_mbc5() {
                    // Enable RAM if lower nibble is 0x0A
                    self.ram_enabled = (value & 0x0F) == 0x0A;
                }
            }
            // ROM Bank Number (0x2000-0x3FFF)
            0x2000..=0x3FFF => {
                if self.is_mbc1() {
                    // MBC1: 5-bit bank number, 0 treated as 1
                    let mut bank = value & 0x1F;
                    if bank == 0 {
                        bank = 1;
                    }
                    self.rom_bank = bank;
                } else if self.is_mbc3() {
                    // MBC3: 7-bit bank number
                    let mut bank = value & 0x7F;
                    if bank == 0 {
                        bank = 1;
                    }
                    self.rom_bank = bank;
                } else if self.is_mbc5() {
                    // MBC5: 8-bit bank number (low byte)
                    self.rom_bank = value;
                }
            }
            // RAM Bank Number / Upper ROM Bank (0x4000-0x5FFF)
            0x4000..=0x5FFF => {
                if self.is_mbc1() {
                    // MBC1: 2-bit value for RAM bank or upper ROM bank bits
                    self.ram_bank = value & 0x03;
                } else if self.is_mbc3() {
                    // MBC3: RAM bank (0-3) or RTC register select
                    self.ram_bank = value & 0x0F;
                }
            }
            // Banking Mode Select (0x6000-0x7FFF)
            0x6000..=0x7FFF => {
                if self.is_mbc1() {
                    // MBC1: 0 = ROM banking, 1 = RAM banking
                    self.banking_mode = value & 0x01;
                }
            }
            // Cartridge RAM (0xA000-0xBFFF)
            0xA000..=0xBFFF => {
                if !self.ram_enabled || self.ram.is_empty() {
                    return;
                }
                
                // MBC3: RAM bank 0-3 (RTC registers 0x08-0x0C not implemented)
                // MBC1: RAM bank depends on banking_mode
                let bank = if self.is_mbc3() {
                    (self.ram_bank & 0x03) as usize
                } else if self.banking_mode == 1 {
                    self.ram_bank as usize
                } else {
                    0
                };
                let bank = bank % self.ram_bank_count();
                let addr = (bank * 0x2000) + ((address as usize) - 0xA000);
                
                if addr < self.ram.len() {
                    self.ram[addr] = value;
                    self.need_save = true;
                }
            }
            _ => {}
        }
    }

    /// Check if this is an MBC1 cartridge
    fn is_mbc1(&self) -> bool {
        matches!(self.header.cart_type, 0x01..=0x03)
    }

    /// Check if this is an MBC3 cartridge
    fn is_mbc3(&self) -> bool {
        matches!(self.header.cart_type, 0x0F..=0x13)
    }

    /// Check if this is an MBC5 cartridge
    fn is_mbc5(&self) -> bool {
        matches!(self.header.cart_type, 0x19..=0x1E)
    }

    /// Get save file path
    fn save_path(&self) -> String {
        format!("{}.sav", self.filename)
    }

    /// Load battery save from file
    fn load_battery_save(&mut self) {
        let save_path = self.save_path();
        if let Ok(mut file) = fs::File::open(&save_path) {
            let _ = file.read_exact(&mut self.ram);
            println!("Loaded save file: {}", save_path);
        }
    }

    /// Save battery backup to file
    pub fn save_battery(&mut self) -> io::Result<()> {
        if !self.battery || !self.need_save {
            return Ok(());
        }
        
        let save_path = self.save_path();
        let mut file = fs::File::create(&save_path)?;
        file.write_all(&self.ram)?;
        self.need_save = false;
        println!("Saved to: {}", save_path);
        Ok(())
    }

    /// Check if save is needed
    pub fn needs_save(&self) -> bool {
        self.battery && self.need_save
    }
}

impl Drop for Cartridge {
    fn drop(&mut self) {
        if self.needs_save() {
            let _ = self.save_battery();
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_rom() -> Vec<Byte> {
        let mut rom = vec![0u8; 0x8000]; // 32KB ROM
        
        // Set up header
        // Title
        let title = b"TEST ROM";
        for (i, &b) in title.iter().enumerate() {
            rom[HEADER_TITLE_START + i] = b;
        }
        
        // Cart type: ROM only
        rom[HEADER_CART_TYPE] = 0x00;
        // ROM size: 32KB
        rom[HEADER_ROM_SIZE] = 0x00;
        // RAM size: None
        rom[HEADER_RAM_SIZE] = 0x00;
        // License code
        rom[HEADER_LIC_CODE] = 0x00;
        // Version
        rom[HEADER_VERSION] = 0x00;
        
        // Calculate and set checksum
        rom[HEADER_CHECKSUM] = Cartridge::calculate_checksum(&rom);
        
        rom
    }

    #[test]
    fn test_header_parse() {
        let rom = create_test_rom();
        let header = RomHeader::parse(&rom).unwrap();
        
        assert_eq!(header.title, "TEST ROM");
        assert_eq!(header.cart_type, 0x00);
        assert_eq!(header.rom_size, 0x00);
        assert_eq!(header.ram_size, 0x00);
    }

    #[test]
    fn test_checksum_calculation() {
        let rom = create_test_rom();
        assert!(Cartridge::validate_checksum(&rom));
    }

    #[test]
    fn test_rom_size_bytes() {
        let rom = create_test_rom();
        let header = RomHeader::parse(&rom).unwrap();
        assert_eq!(header.rom_size_bytes(), 32768);
    }

    #[test]
    fn test_cart_type_name() {
        let rom = create_test_rom();
        let header = RomHeader::parse(&rom).unwrap();
        assert_eq!(header.cart_type_name(), "ROM ONLY");
    }
}
