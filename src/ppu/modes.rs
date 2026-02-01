//! PPU Modes
//!
//! This module implements PPU mode transitions and timing.

// TODO: Implement in task 15.2
// - PpuMode enum (OamSearch, Transfer, HBlank, VBlank)
// - Mode timing:
//   - OAM Search (mode 2): 80 T-cycles
//   - Pixel Transfer (mode 3): variable
//   - HBlank (mode 0): until 456 T-cycles per line
//   - VBlank (mode 1): scanlines 144-153
