//! APU Module
//!
//! This module implements the Audio Processing Unit (APU) for the Game Boy.
//! The APU generates audio through 4 channels:
//! - Channel 1: Square wave with sweep
//! - Channel 2: Square wave
//! - Channel 3: Wave
//! - Channel 4: Noise

pub mod channels;
pub mod mixer;

use crate::common::Byte;
use channels::{Channel1, Channel2, Channel3, Channel4};

/// Audio sample rate
pub const SAMPLE_RATE: u32 = 44100;
/// CPU clock frequency
pub const CPU_CLOCK: u32 = 4194304;
/// Samples per frame sequencer tick (512 Hz)
pub const FRAME_SEQUENCER_RATE: u32 = 8192;

/// Audio Processing Unit
#[derive(Debug)]
pub struct Apu {
    /// Channel 1 (square wave with sweep)
    pub ch1: Channel1,
    /// Channel 2 (square wave)
    pub ch2: Channel2,
    /// Channel 3 (wave)
    pub ch3: Channel3,
    /// Channel 4 (noise)
    pub ch4: Channel4,
    /// NR50 - Master volume & VIN panning
    pub nr50: Byte,
    /// NR51 - Sound panning
    pub nr51: Byte,
    /// NR52 - Sound on/off
    pub nr52: Byte,
    /// Frame sequencer timer
    frame_sequencer_timer: u32,
    /// Frame sequencer step (0-7)
    frame_sequencer_step: u8,
    /// Sample timer for audio output
    sample_timer: u32,
    /// Audio buffer
    pub audio_buffer: Vec<i16>,
    /// Buffer write position
    buffer_pos: usize,
    /// APU enabled
    enabled: bool,
}

impl Default for Apu {
    fn default() -> Self {
        Self::new()
    }
}

impl Apu {
    /// Create a new APU
    pub fn new() -> Self {
        Self {
            ch1: Channel1::new(),
            ch2: Channel2::new(),
            ch3: Channel3::new(),
            ch4: Channel4::new(),
            nr50: 0x77,
            nr51: 0xF3,
            nr52: 0xF1,
            frame_sequencer_timer: 0,
            frame_sequencer_step: 0,
            sample_timer: 0,
            audio_buffer: vec![0; 4096],
            buffer_pos: 0,
            enabled: true,
        }
    }

    /// Initialize APU
    pub fn init(&mut self) {
        self.ch1 = Channel1::new();
        self.ch2 = Channel2::new();
        self.ch3 = Channel3::new();
        self.ch4 = Channel4::new();
        self.nr50 = 0x77;
        self.nr51 = 0xF3;
        self.nr52 = 0xF1;
        self.frame_sequencer_timer = 0;
        self.frame_sequencer_step = 0;
        self.sample_timer = 0;
        self.buffer_pos = 0;
        self.enabled = true;
    }

    /// Tick APU by one T-cycle
    pub fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        // Tick frame sequencer
        self.frame_sequencer_timer += 1;
        if self.frame_sequencer_timer >= FRAME_SEQUENCER_RATE {
            self.frame_sequencer_timer = 0;
            self.tick_frame_sequencer();
        }

        // Tick channels
        self.ch1.tick();
        self.ch2.tick();
        self.ch3.tick();
        self.ch4.tick();

        // Generate sample
        self.sample_timer += SAMPLE_RATE;
        if self.sample_timer >= CPU_CLOCK {
            self.sample_timer -= CPU_CLOCK;
            self.generate_sample();
        }
    }

    /// Tick frame sequencer (512 Hz, 8 steps)
    fn tick_frame_sequencer(&mut self) {
        match self.frame_sequencer_step {
            0 => {
                // Length counter
                self.ch1.tick_length();
                self.ch2.tick_length();
                self.ch3.tick_length();
                self.ch4.tick_length();
            }
            2 => {
                // Length counter + Sweep
                self.ch1.tick_length();
                self.ch2.tick_length();
                self.ch3.tick_length();
                self.ch4.tick_length();
                self.ch1.tick_sweep();
            }
            4 => {
                // Length counter
                self.ch1.tick_length();
                self.ch2.tick_length();
                self.ch3.tick_length();
                self.ch4.tick_length();
            }
            6 => {
                // Length counter + Sweep
                self.ch1.tick_length();
                self.ch2.tick_length();
                self.ch3.tick_length();
                self.ch4.tick_length();
                self.ch1.tick_sweep();
            }
            7 => {
                // Volume envelope
                self.ch1.tick_envelope();
                self.ch2.tick_envelope();
                self.ch4.tick_envelope();
            }
            _ => {}
        }

        self.frame_sequencer_step = (self.frame_sequencer_step + 1) & 7;
    }

    /// Generate audio sample
    fn generate_sample(&mut self) {
        if self.buffer_pos >= self.audio_buffer.len() {
            return;
        }

        let mut left: i32 = 0;
        let mut right: i32 = 0;

        // Get channel outputs
        let ch1_out = self.ch1.output() as i32;
        let ch2_out = self.ch2.output() as i32;
        let ch3_out = self.ch3.output() as i32;
        let ch4_out = self.ch4.output() as i32;

        // Mix channels based on NR51 panning
        if self.nr51 & 0x10 != 0 { left += ch1_out; }
        if self.nr51 & 0x20 != 0 { left += ch2_out; }
        if self.nr51 & 0x40 != 0 { left += ch3_out; }
        if self.nr51 & 0x80 != 0 { left += ch4_out; }
        if self.nr51 & 0x01 != 0 { right += ch1_out; }
        if self.nr51 & 0x02 != 0 { right += ch2_out; }
        if self.nr51 & 0x04 != 0 { right += ch3_out; }
        if self.nr51 & 0x08 != 0 { right += ch4_out; }

        // Apply master volume
        let left_vol = ((self.nr50 >> 4) & 0x07) as i32 + 1;
        let right_vol = (self.nr50 & 0x07) as i32 + 1;

        left = (left * left_vol) / 4;
        right = (right * right_vol) / 4;

        // Scale to i16 range
        left = (left * 256).clamp(-32768, 32767);
        right = (right * 256).clamp(-32768, 32767);

        // Write stereo sample
        if self.buffer_pos + 1 < self.audio_buffer.len() {
            self.audio_buffer[self.buffer_pos] = left as i16;
            self.audio_buffer[self.buffer_pos + 1] = right as i16;
            self.buffer_pos += 2;
        }
    }

    /// Get audio buffer and reset position
    pub fn get_audio_buffer(&mut self) -> &[i16] {
        let len = self.buffer_pos;
        self.buffer_pos = 0;
        &self.audio_buffer[..len]
    }

    /// Read APU register
    pub fn read(&self, address: u16) -> Byte {
        match address {
            // Channel 1
            0xFF10 => self.ch1.read_nr10(),
            0xFF11 => self.ch1.read_nr11(),
            0xFF12 => self.ch1.read_nr12(),
            0xFF13 => 0xFF, // NR13 write-only
            0xFF14 => self.ch1.read_nr14(),
            // Channel 2
            0xFF16 => self.ch2.read_nr21(),
            0xFF17 => self.ch2.read_nr22(),
            0xFF18 => 0xFF, // NR23 write-only
            0xFF19 => self.ch2.read_nr24(),
            // Channel 3
            0xFF1A => self.ch3.read_nr30(),
            0xFF1B => 0xFF, // NR31 write-only
            0xFF1C => self.ch3.read_nr32(),
            0xFF1D => 0xFF, // NR33 write-only
            0xFF1E => self.ch3.read_nr34(),
            // Wave RAM
            0xFF30..=0xFF3F => self.ch3.read_wave_ram(address),
            // Channel 4
            0xFF20 => 0xFF, // NR41 write-only
            0xFF21 => self.ch4.read_nr42(),
            0xFF22 => self.ch4.read_nr43(),
            0xFF23 => self.ch4.read_nr44(),
            // Master registers
            0xFF24 => self.nr50,
            0xFF25 => self.nr51,
            0xFF26 => {
                let mut result = self.nr52 & 0x80;
                if self.ch1.enabled { result |= 0x01; }
                if self.ch2.enabled { result |= 0x02; }
                if self.ch3.enabled { result |= 0x04; }
                if self.ch4.enabled { result |= 0x08; }
                result | 0x70 // Bits 4-6 always read as 1
            }
            _ => 0xFF,
        }
    }

    /// Write APU register
    pub fn write(&mut self, address: u16, value: Byte) {
        // If APU is disabled, only NR52 can be written
        if !self.enabled && address != 0xFF26 && !(0xFF30..=0xFF3F).contains(&address) {
            return;
        }

        match address {
            // Channel 1
            0xFF10 => self.ch1.write_nr10(value),
            0xFF11 => self.ch1.write_nr11(value),
            0xFF12 => self.ch1.write_nr12(value),
            0xFF13 => self.ch1.write_nr13(value),
            0xFF14 => self.ch1.write_nr14(value),
            // Channel 2
            0xFF16 => self.ch2.write_nr21(value),
            0xFF17 => self.ch2.write_nr22(value),
            0xFF18 => self.ch2.write_nr23(value),
            0xFF19 => self.ch2.write_nr24(value),
            // Channel 3
            0xFF1A => self.ch3.write_nr30(value),
            0xFF1B => self.ch3.write_nr31(value),
            0xFF1C => self.ch3.write_nr32(value),
            0xFF1D => self.ch3.write_nr33(value),
            0xFF1E => self.ch3.write_nr34(value),
            // Wave RAM
            0xFF30..=0xFF3F => self.ch3.write_wave_ram(address, value),
            // Channel 4
            0xFF20 => self.ch4.write_nr41(value),
            0xFF21 => self.ch4.write_nr42(value),
            0xFF22 => self.ch4.write_nr43(value),
            0xFF23 => self.ch4.write_nr44(value),
            // Master registers
            0xFF24 => self.nr50 = value,
            0xFF25 => self.nr51 = value,
            0xFF26 => {
                let was_enabled = self.enabled;
                self.enabled = (value & 0x80) != 0;
                self.nr52 = value & 0x80;

                // If APU is turned off, reset all registers
                if was_enabled && !self.enabled {
                    self.ch1 = Channel1::new();
                    self.ch2 = Channel2::new();
                    self.ch3 = Channel3::new();
                    self.ch4 = Channel4::new();
                    self.nr50 = 0;
                    self.nr51 = 0;
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apu_new() {
        let apu = Apu::new();
        assert!(apu.enabled);
        assert_eq!(apu.nr50, 0x77);
        assert_eq!(apu.nr51, 0xF3);
    }

    #[test]
    fn test_nr52_read() {
        let apu = Apu::new();
        let nr52 = apu.read(0xFF26);
        // Bits 4-6 always 1, bit 7 = enabled
        assert_eq!(nr52 & 0xF0, 0xF0);
    }

    #[test]
    fn test_apu_disable() {
        let mut apu = Apu::new();
        apu.nr50 = 0x77;
        apu.nr51 = 0xF3;
        
        // Disable APU
        apu.write(0xFF26, 0x00);
        
        assert!(!apu.enabled);
        assert_eq!(apu.nr50, 0);
        assert_eq!(apu.nr51, 0);
    }
}
