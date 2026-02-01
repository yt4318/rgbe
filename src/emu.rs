//! Emulator Core
//!
//! This module contains the main emulator structure that integrates
//! all hardware components and manages the emulation loop.

use crate::apu::Apu;
use crate::bus::{Bus, MemoryBus};
use crate::cart::Cartridge;
use crate::cpu::Cpu;
use crate::dma::Dma;
use crate::gamepad::Gamepad;
use crate::lcd::Lcd;
use crate::ppu::Ppu;
use crate::timer::Timer;
use crate::cpu::InterruptType;

/// Emulator context state
#[derive(Debug, Clone)]
pub struct EmulatorContext {
    /// Emulator is paused
    pub paused: bool,
    /// Emulator is running
    pub running: bool,
    /// Request to terminate
    pub die: bool,
    /// Total T-cycles executed
    pub ticks: u64,
}

impl Default for EmulatorContext {
    fn default() -> Self {
        Self {
            paused: false,
            running: true,
            die: false,
            ticks: 0,
        }
    }
}

/// Main Emulator structure
pub struct Emulator {
    /// Emulator context/state
    pub ctx: EmulatorContext,
    /// CPU
    pub cpu: Cpu,
    /// PPU
    pub ppu: Ppu,
    /// APU
    pub apu: Apu,
    /// Timer
    pub timer: Timer,
    /// DMA controller
    pub dma: Dma,
    /// LCD controller
    pub lcd: Lcd,
    /// Cartridge
    pub cart: Cartridge,
    /// Gamepad
    pub gamepad: Gamepad,
    /// Memory bus
    pub bus: Bus,
}

impl Emulator {
    /// Create a new emulator instance with the given ROM file
    pub fn new(rom_path: &str) -> Result<Self, String> {
        // Load cartridge
        let cart = Cartridge::load(rom_path)
            .map_err(|e| format!("Failed to load ROM: {}", e))?;

        println!("Loaded ROM: {}", cart.header.title);
        println!("Type: {} (0x{:02X})", cart.header.cart_type_name(), cart.header.cart_type);
        println!("ROM Size: {} KB", cart.header.rom_size_bytes() / 1024);
        println!("RAM Size: {} KB", cart.header.ram_size_bytes() / 1024);

        // Create components
        let mut cpu = Cpu::new();
        cpu.init();

        let mut ppu = Ppu::new();
        ppu.init();

        let mut apu = Apu::new();
        apu.init();

        let mut timer = Timer::new();
        timer.init();

        let mut dma = Dma::new();
        dma.init();

        let mut lcd = Lcd::new();
        lcd.init();

        let mut gamepad = Gamepad::new();
        gamepad.init();

        let bus = Bus::new();

        let mut emu = Self {
            ctx: EmulatorContext::default(),
            cpu,
            ppu,
            apu,
            timer,
            dma,
            lcd,
            cart,
            gamepad,
            bus,
        };

        // Load ROM into bus
        emu.bus.load_rom(&emu.cart.rom);

        Ok(emu)
    }

    /// Run one CPU instruction and tick all components
    pub fn step(&mut self) -> bool {
        if self.ctx.paused || !self.ctx.running {
            return true;
        }

        // Handle interrupts
        self.cpu.handle_interrupts(&mut self.bus);

        // Handle delayed IME enable
        if self.cpu.enabling_ime {
            self.cpu.enabling_ime = false;
            self.cpu.ime = true;
        }

        // If halted, just tick components
        if self.cpu.halted {
            self.tick_components(4);
            
            // Check if we should wake from halt
            if self.cpu.interrupts_pending() {
                self.cpu.halted = false;
            }
            return true;
        }

        // Fetch instruction
        self.cpu.fetch_instruction(&self.bus);
        self.cpu.fetch_data(&self.bus);

        // Execute instruction
        self.cpu.execute(&mut self.bus);

        // Tick components (4 T-cycles per M-cycle, simplified)
        self.tick_components(4);

        !self.ctx.die
    }

    /// Tick all components by the given number of T-cycles
    fn tick_components(&mut self, cycles: u32) {
        // Sync VRAM/OAM from Bus to PPU before ticking
        self.ppu.vram.copy_from_slice(&self.bus.vram);
        self.ppu.oam.copy_from_slice(&self.bus.oam);

        for _ in 0..cycles {
            self.ctx.ticks += 1;

            // Tick timer
            self.timer.tick();
            if self.timer.interrupt_requested {
                self.cpu.request_interrupt(InterruptType::Timer);
                self.timer.clear_interrupt();
            }

            // Tick PPU
            self.ppu.tick(&mut self.lcd);
            if self.ppu.vblank_interrupt {
                self.cpu.request_interrupt(InterruptType::VBlank);
                self.ppu.clear_vblank_interrupt();
            }
            if self.lcd.stat_interrupt {
                self.cpu.request_interrupt(InterruptType::LcdStat);
                self.lcd.clear_stat_interrupt();
            }

            // Tick DMA
            if let Some((src, dst)) = self.dma.tick() {
                let value = self.bus.read(src);
                self.bus.oam[(dst - 0xFE00) as usize] = value;
            }

            // Tick APU
            self.apu.tick();
        }

        // Check gamepad interrupt
        if self.gamepad.interrupt_requested {
            self.cpu.request_interrupt(InterruptType::Joypad);
            self.gamepad.clear_interrupt();
        }
    }

    /// Run the emulator for one frame
    pub fn run_frame(&mut self) {
        let start_frame = self.ppu.current_frame;
        while self.ppu.current_frame == start_frame && !self.ctx.die {
            self.step();
        }
    }

    /// Pause the emulator
    pub fn pause(&mut self) {
        self.ctx.paused = true;
    }

    /// Resume the emulator
    pub fn resume(&mut self) {
        self.ctx.paused = false;
    }

    /// Toggle pause state
    pub fn toggle_pause(&mut self) {
        self.ctx.paused = !self.ctx.paused;
    }

    /// Stop the emulator
    pub fn stop(&mut self) {
        self.ctx.die = true;
        self.ctx.running = false;
    }

    /// Get the video buffer for rendering
    pub fn get_video_buffer(&self) -> &[u32] {
        &self.ppu.video_buffer
    }

    /// Get the audio buffer
    pub fn get_audio_buffer(&mut self) -> &[i16] {
        self.apu.get_audio_buffer()
    }

    /// Set button state
    pub fn set_button(&mut self, button: crate::gamepad::Button, pressed: bool) {
        self.gamepad.set_button(button, pressed);
    }

    /// Check if emulator is running
    pub fn is_running(&self) -> bool {
        self.ctx.running && !self.ctx.die
    }

    /// Check if emulator is paused
    pub fn is_paused(&self) -> bool {
        self.ctx.paused
    }

    /// Get current frame number
    pub fn current_frame(&self) -> u32 {
        self.ppu.current_frame
    }

    /// Run the emulator (simple loop without UI)
    pub fn run(&mut self) -> Result<(), String> {
        println!("Starting emulation...");
        println!("Note: This is a headless run. Use with SDL2 UI for graphics.");
        
        // Run for a limited number of frames for testing
        let max_frames = 60;
        while self.is_running() && self.current_frame() < max_frames {
            self.run_frame();
        }
        
        println!("Emulation completed. Frames: {}", self.current_frame());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a valid ROM file, so they're marked as ignored
    // Run with: cargo test -- --ignored

    #[test]
    #[ignore]
    fn test_emulator_creation() {
        let emu = Emulator::new("../roms/cpu_instrs.gb");
        assert!(emu.is_ok());
    }
}
