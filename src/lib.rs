//! Game Boy Emulator Library
//!
//! This library provides a complete Game Boy emulator implementation in Rust.
//! It emulates the Sharp LR35902 CPU, PPU, APU, and all other hardware components.

pub mod common;
pub mod emu;
pub mod cpu;
pub mod bus;
pub mod cart;
pub mod ppu;
pub mod apu;
pub mod lcd;
pub mod timer;
pub mod dma;
pub mod ram;
pub mod gamepad;
pub mod interrupts;
pub mod stack;
pub mod ui;
