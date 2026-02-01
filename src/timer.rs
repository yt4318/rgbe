//! Timer
//!
//! This module implements the Game Boy timer registers and timing logic.
//! 
//! Timer registers:
//! - DIV (0xFF04): Divider register, increments at 16384 Hz (every 256 T-cycles)
//! - TIMA (0xFF05): Timer counter
//! - TMA (0xFF06): Timer modulo (reload value)
//! - TAC (0xFF07): Timer control (enable and frequency select)

use crate::common::Byte;

/// Timer frequencies based on TAC bits 0-1
/// Values are in T-cycles per TIMA increment
const TIMER_FREQUENCIES: [u16; 4] = [
    1024, // 00: 4096 Hz (CPU Clock / 1024)
    16,   // 01: 262144 Hz (CPU Clock / 16)
    64,   // 10: 65536 Hz (CPU Clock / 64)
    256,  // 11: 16384 Hz (CPU Clock / 256)
];

/// Game Boy Timer
#[derive(Debug, Clone)]
pub struct Timer {
    /// DIV register (0xFF04) - upper 8 bits of internal 16-bit counter
    div: u16,
    /// TIMA register (0xFF05) - timer counter
    tima: Byte,
    /// TMA register (0xFF06) - timer modulo (reload value)
    tma: Byte,
    /// TAC register (0xFF07) - timer control
    tac: Byte,
    /// Timer interrupt requested flag
    pub interrupt_requested: bool,
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Timer {
    /// Create a new Timer with default state
    pub fn new() -> Self {
        Self {
            div: 0xABCC, // Initial value after boot ROM
            tima: 0,
            tma: 0,
            tac: 0,
            interrupt_requested: false,
        }
    }

    /// Initialize timer to boot ROM skip state
    pub fn init(&mut self) {
        self.div = 0xABCC;
        self.tima = 0;
        self.tma = 0;
        self.tac = 0;
        self.interrupt_requested = false;
    }

    /// Read timer register
    pub fn read(&self, address: u16) -> Byte {
        match address {
            0xFF04 => (self.div >> 8) as Byte, // DIV - upper 8 bits
            0xFF05 => self.tima,               // TIMA
            0xFF06 => self.tma,                // TMA
            0xFF07 => self.tac,                // TAC
            _ => 0xFF,
        }
    }

    /// Write timer register
    pub fn write(&mut self, address: u16, value: Byte) {
        match address {
            0xFF04 => {
                // Writing any value to DIV resets it to 0
                self.div = 0;
            }
            0xFF05 => {
                // Write to TIMA
                self.tima = value;
            }
            0xFF06 => {
                // Write to TMA
                self.tma = value;
            }
            0xFF07 => {
                // Write to TAC (only lower 3 bits are used)
                self.tac = value & 0x07;
            }
            _ => {}
        }
    }

    /// Check if timer is enabled
    fn timer_enabled(&self) -> bool {
        (self.tac & 0x04) != 0
    }

    /// Get the current timer frequency divider
    fn timer_frequency(&self) -> u16 {
        TIMER_FREQUENCIES[(self.tac & 0x03) as usize]
    }

    /// Tick the timer by one T-cycle
    pub fn tick(&mut self) {
        let prev_div = self.div;
        self.div = self.div.wrapping_add(1);

        // Check if timer is enabled
        if !self.timer_enabled() {
            return;
        }

        // Get the bit position to check for falling edge
        let freq = self.timer_frequency();
        let bit_pos = match freq {
            1024 => 9, // Check bit 9 of DIV
            16 => 3,   // Check bit 3 of DIV
            64 => 5,   // Check bit 5 of DIV
            256 => 7,  // Check bit 7 of DIV
            _ => return,
        };

        // Check for falling edge (bit was 1, now 0)
        let prev_bit = (prev_div >> bit_pos) & 1;
        let curr_bit = (self.div >> bit_pos) & 1;

        if prev_bit == 1 && curr_bit == 0 {
            // Increment TIMA
            let (new_tima, overflow) = self.tima.overflowing_add(1);
            
            if overflow {
                // TIMA overflow - reload from TMA and request interrupt
                self.tima = self.tma;
                self.interrupt_requested = true;
            } else {
                self.tima = new_tima;
            }
        }
    }

    /// Clear the interrupt request flag
    pub fn clear_interrupt(&mut self) {
        self.interrupt_requested = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_new() {
        let timer = Timer::new();
        assert_eq!(timer.div, 0xABCC);
        assert_eq!(timer.tima, 0);
        assert_eq!(timer.tma, 0);
        assert_eq!(timer.tac, 0);
        assert!(!timer.interrupt_requested);
    }

    #[test]
    fn test_div_read() {
        let timer = Timer::new();
        // DIV returns upper 8 bits of internal counter
        assert_eq!(timer.read(0xFF04), 0xAB);
    }

    #[test]
    fn test_div_write_resets() {
        let mut timer = Timer::new();
        timer.write(0xFF04, 0x42); // Any write resets DIV
        assert_eq!(timer.div, 0);
        assert_eq!(timer.read(0xFF04), 0);
    }

    #[test]
    fn test_tima_tma_tac_read_write() {
        let mut timer = Timer::new();
        
        timer.write(0xFF05, 0x12); // TIMA
        timer.write(0xFF06, 0x34); // TMA
        timer.write(0xFF07, 0x05); // TAC (enable + freq 01)
        
        assert_eq!(timer.read(0xFF05), 0x12);
        assert_eq!(timer.read(0xFF06), 0x34);
        assert_eq!(timer.read(0xFF07), 0x05);
    }

    #[test]
    fn test_timer_disabled() {
        let mut timer = Timer::new();
        timer.div = 0;
        timer.tima = 0;
        timer.tac = 0x00; // Timer disabled
        
        // Tick many times - TIMA should not change
        for _ in 0..1000 {
            timer.tick();
        }
        
        assert_eq!(timer.tima, 0);
    }

    #[test]
    fn test_timer_overflow() {
        let mut timer = Timer::new();
        timer.div = 0;
        timer.tima = 0xFF;
        timer.tma = 0x42;
        timer.tac = 0x05; // Enable, freq 01 (16 T-cycles)
        
        // Tick until TIMA overflows
        // With freq 01, TIMA increments every 16 T-cycles
        // We need to trigger a falling edge on bit 3
        for _ in 0..16 {
            timer.tick();
        }
        
        // TIMA should have overflowed and reloaded from TMA
        assert_eq!(timer.tima, 0x42);
        assert!(timer.interrupt_requested);
    }
}
