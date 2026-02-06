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
    /// Gamepad
    pub gamepad: Gamepad,
    /// Memory bus (includes cartridge)
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

        let mut bus = Bus::new();
        bus.load_cartridge(cart);

        // Initialize I/O registers to boot ROM skip values
        // Sound registers
        bus.io_regs[0x10] = 0x80;
        bus.io_regs[0x11] = 0xBF;
        bus.io_regs[0x12] = 0xF3;
        bus.io_regs[0x13] = 0xFF;
        bus.io_regs[0x14] = 0xBF;
        bus.io_regs[0x16] = 0x3F;
        bus.io_regs[0x17] = 0x00;
        bus.io_regs[0x18] = 0xFF;
        bus.io_regs[0x19] = 0xBF;
        bus.io_regs[0x1A] = 0x7F;
        bus.io_regs[0x1B] = 0xFF;
        bus.io_regs[0x1C] = 0x9F;
        bus.io_regs[0x1D] = 0xFF;
        bus.io_regs[0x1E] = 0xBF;
        bus.io_regs[0x20] = 0xFF;
        bus.io_regs[0x21] = 0x00;
        bus.io_regs[0x22] = 0x00;
        bus.io_regs[0x23] = 0xBF;
        bus.io_regs[0x24] = 0x77;
        bus.io_regs[0x25] = 0xF3;
        bus.io_regs[0x26] = 0xF1;

        bus.io_regs[0x40] = lcd.lcdc;  // LCDC
        bus.io_regs[0x41] = lcd.stat;  // STAT
        bus.io_regs[0x47] = lcd.bgp;   // BGP
        bus.io_regs[0x48] = lcd.obp0;  // OBP0
        bus.io_regs[0x49] = lcd.obp1;  // OBP1

        Ok(Self {
            ctx: EmulatorContext::default(),
            cpu,
            ppu,
            apu,
            timer,
            dma,
            lcd,
            gamepad,
            bus,
        })
    }

    /// Run one CPU instruction and tick all components
    pub fn step(&mut self) -> bool {
        if self.ctx.paused || !self.ctx.running {
            return true;
        }

        self.cpu.reset_step_cycles();

        // Sync IE/IF registers from Bus to CPU
        self.cpu.ie_register = self.bus.ie_register;
        self.cpu.int_flags = self.bus.int_flags;

        // Sync LCD registers from Bus to LCD
        self.sync_lcd_from_bus();

        // Sync Timer registers from Bus
        self.sync_timer_from_bus();

        // Sync Gamepad register from Bus
        self.sync_gamepad_from_bus();

        // Sync APU registers from Bus
        self.sync_apu_from_bus();

        // Check if DMA should start
        self.check_dma_start();

        // Handle interrupts
        if self.cpu.handle_interrupts(&mut self.bus) {
            self.cpu.add_m_cycles(5);
            let t_cycles = self.cpu.take_t_cycles();
            self.tick_components(t_cycles);
            return !self.ctx.die;
        }

        // Sync IF back to Bus after interrupt handling
        self.bus.int_flags = self.cpu.int_flags;

        // Handle delayed IME enable
        if self.cpu.enabling_ime {
            self.cpu.enabling_ime = false;
            self.cpu.ime = true;
        }

        // If halted, just tick components
        if self.cpu.halted {
            self.cpu.add_m_cycles(1);
            let t_cycles = self.cpu.take_t_cycles();
            self.tick_components(t_cycles);
            
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

        // CPU instructions may have written IE/IF through the bus.
        // Re-sync Bus -> CPU so interrupt state stays coherent.
        self.cpu.ie_register = self.bus.ie_register;
        self.cpu.int_flags = self.bus.int_flags;

        // CPU may have written I/O registers via the bus. Apply those writes to
        // component state before ticking so effects are visible immediately.
        self.sync_lcd_from_bus();
        self.sync_timer_from_bus();
        self.sync_gamepad_from_bus();
        self.sync_apu_from_bus();
        self.check_dma_start();

        // Tick components based on consumed CPU cycles
        let t_cycles = self.cpu.take_t_cycles();
        if t_cycles == 0 {
            self.tick_components(4);
        } else {
            self.tick_components(t_cycles);
        }

        !self.ctx.die
    }

    /// Sync LCD registers from Bus I/O area
    fn sync_lcd_from_bus(&mut self) {
        self.lcd.lcdc = self.bus.io_regs[0x40];
        self.lcd.stat = (self.lcd.stat & 0x07) | (self.bus.io_regs[0x41] & 0xF8);
        self.lcd.scy = self.bus.io_regs[0x42];
        self.lcd.scx = self.bus.io_regs[0x43];
        // LY is read-only, don't sync from bus
        self.lcd.lyc = self.bus.io_regs[0x45];
        self.lcd.bgp = self.bus.io_regs[0x47];
        self.lcd.obp0 = self.bus.io_regs[0x48];
        self.lcd.obp1 = self.bus.io_regs[0x49];
        self.lcd.wy = self.bus.io_regs[0x4A];
        self.lcd.wx = self.bus.io_regs[0x4B];
    }

    /// Sync Timer registers from Bus I/O area
    fn sync_timer_from_bus(&mut self) {
        // Check if DIV was written (any write resets it)
        // We track this by checking if the value changed to 0
        let bus_div = self.bus.io_regs[0x04];
        if bus_div == 0 && self.timer.read(0xFF04) != 0 {
            self.timer.write(0xFF04, 0); // Reset DIV
        }
        // TIMA, TMA, TAC are synced
        self.timer.write(0xFF05, self.bus.io_regs[0x05]); // TIMA
        self.timer.write(0xFF06, self.bus.io_regs[0x06]); // TMA
        self.timer.write(0xFF07, self.bus.io_regs[0x07]); // TAC
    }

    /// Sync Timer registers to Bus I/O area
    fn sync_timer_to_bus(&mut self) {
        self.bus.io_regs[0x04] = self.timer.read(0xFF04); // DIV
        self.bus.io_regs[0x05] = self.timer.read(0xFF05); // TIMA
        self.bus.io_regs[0x06] = self.timer.read(0xFF06); // TMA
        self.bus.io_regs[0x07] = self.timer.read(0xFF07); // TAC
    }

    /// Sync Gamepad register from Bus I/O area
    fn sync_gamepad_from_bus(&mut self) {
        self.gamepad.write(self.bus.io_regs[0x00]);
    }

    /// Sync Gamepad register to Bus I/O area
    fn sync_gamepad_to_bus(&mut self) {
        self.bus.io_regs[0x00] = self.gamepad.read();
    }

    /// Check and start DMA if requested
    fn check_dma_start(&mut self) {
        if self.bus.take_io_written(0x46) {
            let dma_reg = self.bus.io_regs[0x46];
            self.dma.start(dma_reg);
            self.bus.set_dma_active(true);
        }
    }

    /// Sync APU registers from Bus I/O area
    fn sync_apu_from_bus(&mut self) {
        const APU_IO_REGS: [usize; 21] = [
            0x10, 0x11, 0x12, 0x13, 0x14, // CH1
            0x16, 0x17, 0x18, 0x19, // CH2
            0x1A, 0x1B, 0x1C, 0x1D, 0x1E, // CH3
            0x20, 0x21, 0x22, 0x23, // CH4
            0x24, 0x25, 0x26, // Master
        ];

        for &reg in &APU_IO_REGS {
            if self.bus.take_io_written(reg) {
                let value = self.bus.io_regs[reg];
                self.apu.write(0xFF00 + reg as u16, value);
            }
        }

        // Wave RAM (0xFF30-0xFF3F)
        for reg in 0x30..=0x3F {
            if self.bus.take_io_written(reg) {
                let value = self.bus.io_regs[reg];
                self.apu.write(0xFF00 + reg as u16, value);
            }
        }
    }

    /// Sync APU registers to Bus I/O area
    fn sync_apu_to_bus(&mut self) {
        // Expose status register readback without feeding it back as writes.
        self.bus.io_regs[0x26] = self.apu.read(0xFF26);
    }

    /// Sync LCD registers to Bus I/O area
    fn sync_lcd_to_bus(&mut self) {
        self.bus.io_regs[0x40] = self.lcd.lcdc;
        self.bus.io_regs[0x41] = self.lcd.stat | 0x80;
        self.bus.io_regs[0x42] = self.lcd.scy;
        self.bus.io_regs[0x43] = self.lcd.scx;
        self.bus.io_regs[0x44] = self.lcd.ly;
        self.bus.io_regs[0x45] = self.lcd.lyc;
        self.bus.io_regs[0x47] = self.lcd.bgp;
        self.bus.io_regs[0x48] = self.lcd.obp0;
        self.bus.io_regs[0x49] = self.lcd.obp1;
        self.bus.io_regs[0x4A] = self.lcd.wy;
        self.bus.io_regs[0x4B] = self.lcd.wx;
    }

    /// Tick all components by the given number of T-cycles
    fn tick_components(&mut self, cycles: u32) {
        // Sync VRAM/OAM lazily: only copy when Bus memory actually changed.
        if self.bus.vram_dirty {
            self.ppu.vram.copy_from_slice(&self.bus.vram);
            self.bus.vram_dirty = false;
        }
        if self.bus.oam_dirty {
            self.ppu.oam.copy_from_slice(&self.bus.oam);
            self.bus.oam_dirty = false;
        }

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
                let oam_index = (dst - 0xFE00) as usize;
                self.bus.oam[oam_index] = value;
                self.ppu.oam[oam_index] = value;
            }
            
            // Update DMA active state
            if !self.dma.active {
                self.bus.set_dma_active(false);
            }

            // Tick APU
            self.apu.tick();
        }

        // Check gamepad interrupt
        if self.gamepad.interrupt_requested {
            self.cpu.request_interrupt(InterruptType::Joypad);
            self.gamepad.clear_interrupt();
        }

        // Sync IF register back to Bus
        self.bus.int_flags = self.cpu.int_flags;

        // Sync LCD registers to Bus
        self.sync_lcd_to_bus();

        // Sync Timer registers to Bus
        self.sync_timer_to_bus();

        // Sync Gamepad register to Bus
        self.sync_gamepad_to_bus();

        // Sync DMA register to Bus
        self.bus.io_regs[0x46] = self.dma.read();

        // Sync APU registers to Bus
        self.sync_apu_to_bus();
    }

    /// Run the emulator for one frame
    pub fn run_frame(&mut self) {
        const T_CYCLES_PER_FRAME: u64 = 70224;
        let start_ticks = self.ctx.ticks;
        while self.ctx.ticks.saturating_sub(start_ticks) < T_CYCLES_PER_FRAME && !self.ctx.die {
            if !self.step() {
                break;
            }
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
        
        // Run for a limited number of simulated frames for testing.
        let max_frames = 60;
        let mut simulated_frames = 0;
        while self.is_running() && simulated_frames < max_frames {
            self.run_frame();
            simulated_frames += 1;
        }
        
        println!(
            "Emulation completed. Simulated frames: {}, PPU frames: {}",
            simulated_frames,
            self.current_frame()
        );
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
