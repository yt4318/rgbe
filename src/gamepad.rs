//! Gamepad
//!
//! This module implements Game Boy joypad input handling.
//!
//! JOYP Register (0xFF00):
//! - Bit 5: Select button keys (0 = selected)
//! - Bit 4: Select direction keys (0 = selected)
//! - Bit 3: Down or Start (0 = pressed)
//! - Bit 2: Up or Select (0 = pressed)
//! - Bit 1: Left or B (0 = pressed)
//! - Bit 0: Right or A (0 = pressed)

use crate::common::Byte;

/// Game Boy buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    A,
    B,
    Select,
    Start,
    Right,
    Left,
    Up,
    Down,
}

/// Gamepad state
#[derive(Debug, Clone)]
pub struct Gamepad {
    /// Button states (true = pressed)
    pub button_a: bool,
    pub button_b: bool,
    pub button_select: bool,
    pub button_start: bool,
    pub dpad_right: bool,
    pub dpad_left: bool,
    pub dpad_up: bool,
    pub dpad_down: bool,
    /// Selection register (bits 4-5 of JOYP)
    pub selection: Byte,
    /// Joypad interrupt requested
    pub interrupt_requested: bool,
}

impl Default for Gamepad {
    fn default() -> Self {
        Self::new()
    }
}

impl Gamepad {
    /// Create a new Gamepad
    pub fn new() -> Self {
        Self {
            button_a: false,
            button_b: false,
            button_select: false,
            button_start: false,
            dpad_right: false,
            dpad_left: false,
            dpad_up: false,
            dpad_down: false,
            selection: 0x30, // Both deselected
            interrupt_requested: false,
        }
    }

    /// Initialize gamepad
    pub fn init(&mut self) {
        self.button_a = false;
        self.button_b = false;
        self.button_select = false;
        self.button_start = false;
        self.dpad_right = false;
        self.dpad_left = false;
        self.dpad_up = false;
        self.dpad_down = false;
        self.selection = 0x30;
        self.interrupt_requested = false;
    }

    /// Read JOYP register (0xFF00)
    pub fn read(&self) -> Byte {
        let mut result = 0xCF; // Upper bits always 1, lower 4 bits start as 1 (not pressed)

        // Add selection bits
        result |= self.selection;

        // Check if button keys selected (bit 5 = 0)
        if (self.selection & 0x20) == 0 {
            if self.button_start { result &= !0x08; }
            if self.button_select { result &= !0x04; }
            if self.button_b { result &= !0x02; }
            if self.button_a { result &= !0x01; }
        }

        // Check if direction keys selected (bit 4 = 0)
        if (self.selection & 0x10) == 0 {
            if self.dpad_down { result &= !0x08; }
            if self.dpad_up { result &= !0x04; }
            if self.dpad_left { result &= !0x02; }
            if self.dpad_right { result &= !0x01; }
        }

        result
    }

    /// Write JOYP register (0xFF00)
    /// Only bits 4-5 are writable (selection)
    pub fn write(&mut self, value: Byte) {
        self.selection = value & 0x30;
    }

    /// Set button state
    pub fn set_button(&mut self, button: Button, pressed: bool) {
        let was_pressed = self.is_pressed(button);
        
        match button {
            Button::A => self.button_a = pressed,
            Button::B => self.button_b = pressed,
            Button::Select => self.button_select = pressed,
            Button::Start => self.button_start = pressed,
            Button::Right => self.dpad_right = pressed,
            Button::Left => self.dpad_left = pressed,
            Button::Up => self.dpad_up = pressed,
            Button::Down => self.dpad_down = pressed,
        }

        // Request interrupt on button press (high to low transition)
        if pressed && !was_pressed {
            self.interrupt_requested = true;
        }
    }

    /// Check if button is pressed
    pub fn is_pressed(&self, button: Button) -> bool {
        match button {
            Button::A => self.button_a,
            Button::B => self.button_b,
            Button::Select => self.button_select,
            Button::Start => self.button_start,
            Button::Right => self.dpad_right,
            Button::Left => self.dpad_left,
            Button::Up => self.dpad_up,
            Button::Down => self.dpad_down,
        }
    }

    /// Clear interrupt flag
    pub fn clear_interrupt(&mut self) {
        self.interrupt_requested = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gamepad_new() {
        let gamepad = Gamepad::new();
        assert!(!gamepad.button_a);
        assert!(!gamepad.dpad_up);
        assert_eq!(gamepad.selection, 0x30);
    }

    #[test]
    fn test_joyp_read_no_selection() {
        let gamepad = Gamepad::new();
        // With both deselected (0x30), should read 0xFF
        assert_eq!(gamepad.read(), 0xFF);
    }

    #[test]
    fn test_joyp_read_buttons() {
        let mut gamepad = Gamepad::new();
        gamepad.write(0x10); // Select buttons (bit 5 = 0, bit 4 = 1)
        
        // No buttons pressed
        assert_eq!(gamepad.read() & 0x0F, 0x0F);
        
        // Press A
        gamepad.button_a = true;
        assert_eq!(gamepad.read() & 0x0F, 0x0E);
        
        // Press Start
        gamepad.button_start = true;
        assert_eq!(gamepad.read() & 0x0F, 0x06);
    }

    #[test]
    fn test_joyp_read_directions() {
        let mut gamepad = Gamepad::new();
        gamepad.write(0x20); // Select directions (bit 4 = 0, bit 5 = 1)
        
        // No directions pressed
        assert_eq!(gamepad.read() & 0x0F, 0x0F);
        
        // Press Right
        gamepad.dpad_right = true;
        assert_eq!(gamepad.read() & 0x0F, 0x0E);
        
        // Press Up
        gamepad.dpad_up = true;
        assert_eq!(gamepad.read() & 0x0F, 0x0A);
    }

    #[test]
    fn test_button_interrupt() {
        let mut gamepad = Gamepad::new();
        
        assert!(!gamepad.interrupt_requested);
        
        gamepad.set_button(Button::A, true);
        assert!(gamepad.interrupt_requested);
        
        gamepad.clear_interrupt();
        assert!(!gamepad.interrupt_requested);
        
        // Releasing doesn't trigger interrupt
        gamepad.set_button(Button::A, false);
        assert!(!gamepad.interrupt_requested);
    }
}
