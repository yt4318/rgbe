//! Interrupts
//!
//! This module implements the Game Boy interrupt system.

// TODO: Implement in task 10
// - InterruptType enum (VBlank, LcdStat, Timer, Serial, Joypad)
// - IE register (0xFFFF) - interrupt enable
// - IF register (0xFF0F) - interrupt flags
// - Interrupt handling with priority
// - Interrupt vectors:
//   - VBlank: 0x0040
//   - LCD STAT: 0x0048
//   - Timer: 0x0050
//   - Serial: 0x0058
//   - Joypad: 0x0060
