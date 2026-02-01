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
        self.cpu.handle_interrupts(&mut self.bus);

        // Sync IF back to Bus after interrupt handling
        self.bus.int_flags = self.cpu.int_flags;

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
        let pc_before = self.cpu.regs.pc;
        self.cpu.fetch_instruction(&self.bus);
        self.cpu.fetch_data(&self.bus);

        // Debug output for first 20 instructions
        if self.ctx.ticks < 100 {
            println!("PC:{:04X} OP:{:02X} {:?}", pc_before, self.cpu.cur_opcode, 
                self.cpu.current_instruction().map(|i| i.inst_type));
        }

        // Execute instruction
        self.cpu.execute(&mut self.bus);

        // Sync IE/IF back to Bus after execution
        self.bus.ie_register = self.cpu.ie_register;
        self.bus.int_flags = self.cpu.int_flags;

        // Sync LCD registers back to Bus
        self.sync_lcd_to_bus();

        // Sync Gamepad register back to Bus
        self.sync_gamepad_to_bus();

        // Tick components (4 T-cycles per M-cycle, simplified)
        self.tick_components(4);

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
        let dma_reg = self.bus.io_regs[0x46];
        if dma_reg != self.dma.value && !self.dma.active {
            self.dma.start(dma_reg);
            self.bus.set_dma_active(true);
        }
    }

    /// Sync APU registers from Bus I/O area
    fn sync_apu_from_bus(&mut self) {
        // Channel 1
        self.apu.write(0xFF10, self.bus.io_regs[0x10]);
        self.apu.write(0xFF11, self.bus.io_regs[0x11]);
        self.apu.write(0xFF12, self.bus.io_regs[0x12]);
        self.apu.write(0xFF13, self.bus.io_regs[0x13]);
        self.apu.write(0xFF14, self.bus.io_regs[0x14]);
        // Channel 2
        self.apu.write(0xFF16, self.bus.io_regs[0x16]);
        self.apu.write(0xFF17, self.bus.io_regs[0x17]);
        self.apu.write(0xFF18, self.bus.io_regs[0x18]);
        self.apu.write(0xFF19, self.bus.io_regs[0x19]);
        // Channel 3
        self.apu.write(0xFF1A, self.bus.io_regs[0x1A]);
        self.apu.write(0xFF1B, self.bus.io_regs[0x1B]);
        self.apu.write(0xFF1C, self.bus.io_regs[0x1C]);
        self.apu.write(0xFF1D, self.bus.io_regs[0x1D]);
        self.apu.write(0xFF1E, self.bus.io_regs[0x1E]);
        // Channel 4
        self.apu.write(0xFF20, self.bus.io_regs[0x20]);
        self.apu.write(0xFF21, self.bus.io_regs[0x21]);
        self.apu.write(0xFF22, self.bus.io_regs[0x22]);
        self.apu.write(0xFF23, self.bus.io_regs[0x23]);
        // Master registers
        self.apu.write(0xFF24, self.bus.io_regs[0x24]);
        self.apu.write(0xFF25, self.bus.io_regs[0x25]);
        self.apu.write(0xFF26, self.bus.io_regs[0x26]);
        // Wave RAM (0xFF30-0xFF3F)
        for i in 0..16 {
            self.apu.write(0xFF30 + i, self.bus.io_regs[0x30 + i as usize]);
        }
    }

    /// Sync APU registers to Bus I/O area
    fn sync_apu_to_bus(&mut self) {
        // Channel 1
        self.bus.io_regs[0x10] = self.apu.read(0xFF10);
        self.bus.io_regs[0x11] = self.apu.read(0xFF11);
        self.bus.io_regs[0x12] = self.apu.read(0xFF12);
        self.bus.io_regs[0x14] = self.apu.read(0xFF14);
        // Channel 2
        self.bus.io_regs[0x16] = self.apu.read(0xFF16);
        self.bus.io_regs[0x17] = self.apu.read(0xFF17);
        self.bus.io_regs[0x19] = self.apu.read(0xFF19);
        // Channel 3
        self.bus.io_regs[0x1A] = self.apu.read(0xFF1A);
        self.bus.io_regs[0x1C] = self.apu.read(0xFF1C);
        self.bus.io_regs[0x1E] = self.apu.read(0xFF1E);
        // Channel 4
        self.bus.io_regs[0x21] = self.apu.read(0xFF21);
        self.bus.io_regs[0x22] = self.apu.read(0xFF22);
        self.bus.io_regs[0x23] = self.apu.read(0xFF23);
        // Master registers
        self.bus.io_regs[0x24] = self.apu.read(0xFF24);
        self.bus.io_regs[0x25] = self.apu.read(0xFF25);
        self.bus.io_regs[0x26] = self.apu.read(0xFF26);
        // Wave RAM (0xFF30-0xFF3F)
        for i in 0..16 {
            self.bus.io_regs[0x30 + i as usize] = self.apu.read(0xFF30 + i);
        }
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
