//! APU Channels
//!
//! This module implements the 4 audio channels of the Game Boy APU.

use crate::common::Byte;

/// Duty cycle patterns (8 steps each)
const DUTY_PATTERNS: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1], // 12.5%
    [1, 0, 0, 0, 0, 0, 0, 1], // 25%
    [1, 0, 0, 0, 0, 1, 1, 1], // 50%
    [0, 1, 1, 1, 1, 1, 1, 0], // 75%
];

/// Channel 1 - Square wave with sweep
#[derive(Debug, Clone)]
pub struct Channel1 {
    pub enabled: bool,
    pub dac_enabled: bool,
    // NR10 - Sweep
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    sweep_timer: u8,
    sweep_enabled: bool,
    sweep_shadow: u16,
    // NR11 - Length/Duty
    duty: u8,
    length_counter: u16,
    // NR12 - Volume envelope
    volume: u8,
    volume_initial: u8,
    envelope_add: bool,
    envelope_period: u8,
    envelope_timer: u8,
    // NR13/NR14 - Frequency
    frequency: u16,
    length_enabled: bool,
    // Internal
    timer: u16,
    duty_position: u8,
}

impl Default for Channel1 {
    fn default() -> Self {
        Self::new()
    }
}

impl Channel1 {
    pub fn new() -> Self {
        Self {
            enabled: false,
            dac_enabled: false,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
            sweep_timer: 0,
            sweep_enabled: false,
            sweep_shadow: 0,
            duty: 0,
            length_counter: 0,
            volume: 0,
            volume_initial: 0,
            envelope_add: false,
            envelope_period: 0,
            envelope_timer: 0,
            frequency: 0,
            length_enabled: false,
            timer: 0,
            duty_position: 0,
        }
    }

    pub fn tick(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
        }
        if self.timer == 0 {
            self.timer = (2048 - self.frequency) * 4;
            self.duty_position = (self.duty_position + 1) & 7;
        }
    }

    pub fn tick_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }

    pub fn tick_envelope(&mut self) {
        if self.envelope_period == 0 {
            return;
        }
        if self.envelope_timer > 0 {
            self.envelope_timer -= 1;
        }
        if self.envelope_timer == 0 {
            self.envelope_timer = self.envelope_period;
            if self.envelope_add && self.volume < 15 {
                self.volume += 1;
            } else if !self.envelope_add && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }

    pub fn tick_sweep(&mut self) {
        if self.sweep_timer > 0 {
            self.sweep_timer -= 1;
        }
        if self.sweep_timer == 0 {
            self.sweep_timer = if self.sweep_period > 0 { self.sweep_period } else { 8 };
            if self.sweep_enabled && self.sweep_period > 0 {
                let new_freq = self.calculate_sweep();
                if new_freq <= 2047 && self.sweep_shift > 0 {
                    self.frequency = new_freq;
                    self.sweep_shadow = new_freq;
                    // Overflow check
                    self.calculate_sweep();
                }
            }
        }
    }

    fn calculate_sweep(&mut self) -> u16 {
        let mut new_freq = self.sweep_shadow >> self.sweep_shift;
        if self.sweep_negate {
            new_freq = self.sweep_shadow.wrapping_sub(new_freq);
        } else {
            new_freq = self.sweep_shadow.wrapping_add(new_freq);
        }
        if new_freq > 2047 {
            self.enabled = false;
        }
        new_freq
    }

    pub fn output(&self) -> u8 {
        if !self.enabled || !self.dac_enabled {
            return 0;
        }
        DUTY_PATTERNS[self.duty as usize][self.duty_position as usize] * self.volume
    }

    fn trigger(&mut self) {
        self.enabled = self.dac_enabled;
        if self.length_counter == 0 {
            self.length_counter = 64;
        }
        self.timer = (2048 - self.frequency) * 4;
        self.envelope_timer = self.envelope_period;
        self.volume = self.volume_initial;
        self.sweep_shadow = self.frequency;
        self.sweep_timer = if self.sweep_period > 0 { self.sweep_period } else { 8 };
        self.sweep_enabled = self.sweep_period > 0 || self.sweep_shift > 0;
        if self.sweep_shift > 0 {
            self.calculate_sweep();
        }
    }

    // Register accessors
    pub fn read_nr10(&self) -> Byte {
        0x80 | (self.sweep_period << 4) | (if self.sweep_negate { 0x08 } else { 0 }) | self.sweep_shift
    }
    pub fn write_nr10(&mut self, value: Byte) {
        self.sweep_period = (value >> 4) & 0x07;
        self.sweep_negate = (value & 0x08) != 0;
        self.sweep_shift = value & 0x07;
    }
    pub fn read_nr11(&self) -> Byte { (self.duty << 6) | 0x3F }
    pub fn write_nr11(&mut self, value: Byte) {
        self.duty = (value >> 6) & 0x03;
        self.length_counter = 64 - (value & 0x3F) as u16;
    }
    pub fn read_nr12(&self) -> Byte {
        (self.volume_initial << 4) | (if self.envelope_add { 0x08 } else { 0 }) | self.envelope_period
    }
    pub fn write_nr12(&mut self, value: Byte) {
        self.volume_initial = (value >> 4) & 0x0F;
        self.envelope_add = (value & 0x08) != 0;
        self.envelope_period = value & 0x07;
        self.dac_enabled = (value & 0xF8) != 0;
        if !self.dac_enabled { self.enabled = false; }
    }
    pub fn write_nr13(&mut self, value: Byte) {
        self.frequency = (self.frequency & 0x700) | value as u16;
    }
    pub fn read_nr14(&self) -> Byte { (if self.length_enabled { 0x40 } else { 0 }) | 0xBF }
    pub fn write_nr14(&mut self, value: Byte) {
        self.length_enabled = (value & 0x40) != 0;
        self.frequency = (self.frequency & 0xFF) | ((value as u16 & 0x07) << 8);
        if (value & 0x80) != 0 { self.trigger(); }
    }
}


/// Channel 2 - Square wave (no sweep)
#[derive(Debug, Clone)]
pub struct Channel2 {
    pub enabled: bool,
    pub dac_enabled: bool,
    duty: u8,
    length_counter: u16,
    volume: u8,
    volume_initial: u8,
    envelope_add: bool,
    envelope_period: u8,
    envelope_timer: u8,
    frequency: u16,
    length_enabled: bool,
    timer: u16,
    duty_position: u8,
}

impl Default for Channel2 {
    fn default() -> Self { Self::new() }
}

impl Channel2 {
    pub fn new() -> Self {
        Self {
            enabled: false, dac_enabled: false, duty: 0, length_counter: 0,
            volume: 0, volume_initial: 0, envelope_add: false, envelope_period: 0,
            envelope_timer: 0, frequency: 0, length_enabled: false, timer: 0, duty_position: 0,
        }
    }

    pub fn tick(&mut self) {
        if self.timer > 0 { self.timer -= 1; }
        if self.timer == 0 {
            self.timer = (2048 - self.frequency) * 4;
            self.duty_position = (self.duty_position + 1) & 7;
        }
    }

    pub fn tick_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 { self.enabled = false; }
        }
    }

    pub fn tick_envelope(&mut self) {
        if self.envelope_period == 0 { return; }
        if self.envelope_timer > 0 { self.envelope_timer -= 1; }
        if self.envelope_timer == 0 {
            self.envelope_timer = self.envelope_period;
            if self.envelope_add && self.volume < 15 { self.volume += 1; }
            else if !self.envelope_add && self.volume > 0 { self.volume -= 1; }
        }
    }

    pub fn output(&self) -> u8 {
        if !self.enabled || !self.dac_enabled { return 0; }
        DUTY_PATTERNS[self.duty as usize][self.duty_position as usize] * self.volume
    }

    fn trigger(&mut self) {
        self.enabled = self.dac_enabled;
        if self.length_counter == 0 { self.length_counter = 64; }
        self.timer = (2048 - self.frequency) * 4;
        self.envelope_timer = self.envelope_period;
        self.volume = self.volume_initial;
    }

    pub fn read_nr21(&self) -> Byte { (self.duty << 6) | 0x3F }
    pub fn write_nr21(&mut self, value: Byte) {
        self.duty = (value >> 6) & 0x03;
        self.length_counter = 64 - (value & 0x3F) as u16;
    }
    pub fn read_nr22(&self) -> Byte {
        (self.volume_initial << 4) | (if self.envelope_add { 0x08 } else { 0 }) | self.envelope_period
    }
    pub fn write_nr22(&mut self, value: Byte) {
        self.volume_initial = (value >> 4) & 0x0F;
        self.envelope_add = (value & 0x08) != 0;
        self.envelope_period = value & 0x07;
        self.dac_enabled = (value & 0xF8) != 0;
        if !self.dac_enabled { self.enabled = false; }
    }
    pub fn write_nr23(&mut self, value: Byte) { self.frequency = (self.frequency & 0x700) | value as u16; }
    pub fn read_nr24(&self) -> Byte { (if self.length_enabled { 0x40 } else { 0 }) | 0xBF }
    pub fn write_nr24(&mut self, value: Byte) {
        self.length_enabled = (value & 0x40) != 0;
        self.frequency = (self.frequency & 0xFF) | ((value as u16 & 0x07) << 8);
        if (value & 0x80) != 0 { self.trigger(); }
    }
}


/// Channel 3 - Wave
#[derive(Debug, Clone)]
pub struct Channel3 {
    pub enabled: bool,
    pub dac_enabled: bool,
    length_counter: u16,
    volume_code: u8,
    frequency: u16,
    length_enabled: bool,
    wave_ram: [Byte; 16],
    timer: u16,
    wave_position: u8,
}

impl Default for Channel3 {
    fn default() -> Self { Self::new() }
}

impl Channel3 {
    pub fn new() -> Self {
        Self {
            enabled: false, dac_enabled: false, length_counter: 0, volume_code: 0,
            frequency: 0, length_enabled: false, wave_ram: [0; 16], timer: 0, wave_position: 0,
        }
    }

    pub fn tick(&mut self) {
        if self.timer > 0 { self.timer -= 1; }
        if self.timer == 0 {
            self.timer = (2048 - self.frequency) * 2;
            self.wave_position = (self.wave_position + 1) & 31;
        }
    }

    pub fn tick_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 { self.enabled = false; }
        }
    }

    pub fn output(&self) -> u8 {
        if !self.enabled || !self.dac_enabled { return 0; }
        let sample = self.wave_ram[(self.wave_position / 2) as usize];
        let sample = if self.wave_position & 1 == 0 { sample >> 4 } else { sample & 0x0F };
        let shift = match self.volume_code { 0 => 4, 1 => 0, 2 => 1, 3 => 2, _ => 4 };
        sample >> shift
    }

    fn trigger(&mut self) {
        self.enabled = self.dac_enabled;
        if self.length_counter == 0 { self.length_counter = 256; }
        self.timer = (2048 - self.frequency) * 2;
        self.wave_position = 0;
    }

    pub fn read_nr30(&self) -> Byte { (if self.dac_enabled { 0x80 } else { 0 }) | 0x7F }
    pub fn write_nr30(&mut self, value: Byte) {
        self.dac_enabled = (value & 0x80) != 0;
        if !self.dac_enabled { self.enabled = false; }
    }
    pub fn write_nr31(&mut self, value: Byte) { self.length_counter = 256 - value as u16; }
    pub fn read_nr32(&self) -> Byte { (self.volume_code << 5) | 0x9F }
    pub fn write_nr32(&mut self, value: Byte) { self.volume_code = (value >> 5) & 0x03; }
    pub fn write_nr33(&mut self, value: Byte) { self.frequency = (self.frequency & 0x700) | value as u16; }
    pub fn read_nr34(&self) -> Byte { (if self.length_enabled { 0x40 } else { 0 }) | 0xBF }
    pub fn write_nr34(&mut self, value: Byte) {
        self.length_enabled = (value & 0x40) != 0;
        self.frequency = (self.frequency & 0xFF) | ((value as u16 & 0x07) << 8);
        if (value & 0x80) != 0 { self.trigger(); }
    }
    pub fn read_wave_ram(&self, address: u16) -> Byte { self.wave_ram[(address - 0xFF30) as usize] }
    pub fn write_wave_ram(&mut self, address: u16, value: Byte) { self.wave_ram[(address - 0xFF30) as usize] = value; }
}


/// Channel 4 - Noise
#[derive(Debug, Clone)]
pub struct Channel4 {
    pub enabled: bool,
    pub dac_enabled: bool,
    length_counter: u16,
    volume: u8,
    volume_initial: u8,
    envelope_add: bool,
    envelope_period: u8,
    envelope_timer: u8,
    clock_shift: u8,
    width_mode: bool,
    divisor_code: u8,
    length_enabled: bool,
    timer: u16,
    lfsr: u16,
}

impl Default for Channel4 {
    fn default() -> Self { Self::new() }
}

impl Channel4 {
    pub fn new() -> Self {
        Self {
            enabled: false, dac_enabled: false, length_counter: 0, volume: 0,
            volume_initial: 0, envelope_add: false, envelope_period: 0, envelope_timer: 0,
            clock_shift: 0, width_mode: false, divisor_code: 0, length_enabled: false,
            timer: 0, lfsr: 0x7FFF,
        }
    }

    pub fn tick(&mut self) {
        if self.timer > 0 { self.timer -= 1; }
        if self.timer == 0 {
            self.timer = self.get_timer_period();
            let xor_result = (self.lfsr & 1) ^ ((self.lfsr >> 1) & 1);
            self.lfsr = (self.lfsr >> 1) | (xor_result << 14);
            if self.width_mode {
                self.lfsr &= !(1 << 6);
                self.lfsr |= xor_result << 6;
            }
        }
    }

    fn get_timer_period(&self) -> u16 {
        let divisor = match self.divisor_code { 0 => 8, n => (n as u16) * 16 };
        divisor << self.clock_shift
    }

    pub fn tick_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 { self.enabled = false; }
        }
    }

    pub fn tick_envelope(&mut self) {
        if self.envelope_period == 0 { return; }
        if self.envelope_timer > 0 { self.envelope_timer -= 1; }
        if self.envelope_timer == 0 {
            self.envelope_timer = self.envelope_period;
            if self.envelope_add && self.volume < 15 { self.volume += 1; }
            else if !self.envelope_add && self.volume > 0 { self.volume -= 1; }
        }
    }

    pub fn output(&self) -> u8 {
        if !self.enabled || !self.dac_enabled { return 0; }
        if (self.lfsr & 1) == 0 { self.volume } else { 0 }
    }

    fn trigger(&mut self) {
        self.enabled = self.dac_enabled;
        if self.length_counter == 0 { self.length_counter = 64; }
        self.timer = self.get_timer_period();
        self.envelope_timer = self.envelope_period;
        self.volume = self.volume_initial;
        self.lfsr = 0x7FFF;
    }

    pub fn write_nr41(&mut self, value: Byte) { self.length_counter = 64 - (value & 0x3F) as u16; }
    pub fn read_nr42(&self) -> Byte {
        (self.volume_initial << 4) | (if self.envelope_add { 0x08 } else { 0 }) | self.envelope_period
    }
    pub fn write_nr42(&mut self, value: Byte) {
        self.volume_initial = (value >> 4) & 0x0F;
        self.envelope_add = (value & 0x08) != 0;
        self.envelope_period = value & 0x07;
        self.dac_enabled = (value & 0xF8) != 0;
        if !self.dac_enabled { self.enabled = false; }
    }
    pub fn read_nr43(&self) -> Byte {
        (self.clock_shift << 4) | (if self.width_mode { 0x08 } else { 0 }) | self.divisor_code
    }
    pub fn write_nr43(&mut self, value: Byte) {
        self.clock_shift = (value >> 4) & 0x0F;
        self.width_mode = (value & 0x08) != 0;
        self.divisor_code = value & 0x07;
    }
    pub fn read_nr44(&self) -> Byte { (if self.length_enabled { 0x40 } else { 0 }) | 0xBF }
    pub fn write_nr44(&mut self, value: Byte) {
        self.length_enabled = (value & 0x40) != 0;
        if (value & 0x80) != 0 { self.trigger(); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duty_patterns() {
        // 12.5% duty
        assert_eq!(DUTY_PATTERNS[0].iter().filter(|&&x| x == 1).count(), 1);
        // 25% duty
        assert_eq!(DUTY_PATTERNS[1].iter().filter(|&&x| x == 1).count(), 2);
        // 50% duty
        assert_eq!(DUTY_PATTERNS[2].iter().filter(|&&x| x == 1).count(), 4);
        // 75% duty
        assert_eq!(DUTY_PATTERNS[3].iter().filter(|&&x| x == 1).count(), 6);
    }

    #[test]
    fn test_channel1_new() {
        let ch = Channel1::new();
        assert!(!ch.enabled);
        assert!(!ch.dac_enabled);
    }

    #[test]
    fn test_channel4_lfsr() {
        let mut ch = Channel4::new();
        ch.enabled = true;
        ch.dac_enabled = true;
        ch.volume = 15;
        ch.timer = 1;
        
        let initial_lfsr = ch.lfsr;
        ch.tick();
        assert_ne!(ch.lfsr, initial_lfsr);
    }
}
