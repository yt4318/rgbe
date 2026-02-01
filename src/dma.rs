//! DMA Transfer
//!
//! This module implements OAM DMA transfer for the Game Boy.
//! DMA transfers 160 bytes from source address to OAM (0xFE00-0xFE9F).

use crate::common::Byte;

/// DMA Transfer Controller
#[derive(Debug, Clone)]
pub struct Dma {
    /// DMA is currently active
    pub active: bool,
    /// Current byte being transferred (0-159)
    pub byte: u8,
    /// Source address high byte (value written to 0xFF46)
    pub value: Byte,
    /// Delay counter (2 T-cycles before transfer starts)
    pub delay: u8,
}

impl Default for Dma {
    fn default() -> Self {
        Self::new()
    }
}

impl Dma {
    /// Create a new DMA controller
    pub fn new() -> Self {
        Self {
            active: false,
            byte: 0,
            value: 0,
            delay: 0,
        }
    }

    /// Initialize DMA
    pub fn init(&mut self) {
        self.active = false;
        self.byte = 0;
        self.value = 0;
        self.delay = 0;
    }

    /// Start DMA transfer
    /// 
    /// Called when writing to 0xFF46
    pub fn start(&mut self, value: Byte) {
        self.value = value;
        self.active = true;
        self.byte = 0;
        self.delay = 2; // 2 T-cycle delay before transfer starts
    }

    /// Get source address for current byte
    pub fn source_address(&self) -> u16 {
        (self.value as u16) << 8 | (self.byte as u16)
    }

    /// Get destination address for current byte
    pub fn dest_address(&self) -> u16 {
        0xFE00 + self.byte as u16
    }

    /// Tick DMA by one T-cycle
    /// 
    /// Returns Some((source, dest)) if a byte should be transferred this cycle
    pub fn tick(&mut self) -> Option<(u16, u16)> {
        if !self.active {
            return None;
        }

        // Handle delay
        if self.delay > 0 {
            self.delay -= 1;
            return None;
        }

        // Transfer one byte
        let source = self.source_address();
        let dest = self.dest_address();

        self.byte += 1;

        // Check if transfer is complete
        if self.byte >= 160 {
            self.active = false;
        }

        Some((source, dest))
    }

    /// Check if DMA is transferring
    pub fn is_transferring(&self) -> bool {
        self.active && self.delay == 0
    }

    /// Read DMA register (returns last written value)
    pub fn read(&self) -> Byte {
        self.value
    }

    /// Write DMA register (starts transfer)
    pub fn write(&mut self, value: Byte) {
        self.start(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dma_new() {
        let dma = Dma::new();
        assert!(!dma.active);
        assert_eq!(dma.byte, 0);
        assert_eq!(dma.value, 0);
    }

    #[test]
    fn test_dma_start() {
        let mut dma = Dma::new();
        dma.start(0xC0); // Start transfer from 0xC000
        
        assert!(dma.active);
        assert_eq!(dma.value, 0xC0);
        assert_eq!(dma.byte, 0);
        assert_eq!(dma.delay, 2);
    }

    #[test]
    fn test_dma_addresses() {
        let mut dma = Dma::new();
        dma.start(0xC0);
        
        assert_eq!(dma.source_address(), 0xC000);
        assert_eq!(dma.dest_address(), 0xFE00);
        
        dma.byte = 50;
        assert_eq!(dma.source_address(), 0xC032);
        assert_eq!(dma.dest_address(), 0xFE32);
    }

    #[test]
    fn test_dma_tick_delay() {
        let mut dma = Dma::new();
        dma.start(0xC0);
        
        // First two ticks should be delay
        assert!(dma.tick().is_none());
        assert!(dma.tick().is_none());
        
        // Third tick should transfer
        let result = dma.tick();
        assert!(result.is_some());
        let (src, dst) = result.unwrap();
        assert_eq!(src, 0xC000);
        assert_eq!(dst, 0xFE00);
    }

    #[test]
    fn test_dma_complete_transfer() {
        let mut dma = Dma::new();
        dma.start(0xC0);
        
        // Skip delay
        dma.tick();
        dma.tick();
        
        // Transfer 160 bytes
        for i in 0..160 {
            let result = dma.tick();
            assert!(result.is_some(), "Byte {} should transfer", i);
        }
        
        // Should be complete
        assert!(!dma.active);
        assert!(dma.tick().is_none());
    }
}
