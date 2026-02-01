//! Common types and utilities for the Game Boy emulator
//!
//! This module defines type aliases matching Game Boy hardware specifications
//! and provides bit manipulation utilities.

/// 8-bit unsigned integer (Game Boy byte)
pub type Byte = u8;

/// 16-bit unsigned integer (Game Boy word)
pub type Word = u16;

/// 32-bit unsigned integer (double word)
pub type DWord = u32;

/// 64-bit unsigned integer (quad word)
pub type QWord = u64;

/// Check if a specific bit is set in a byte value
///
/// # Arguments
/// * `value` - The byte value to check
/// * `n` - The bit position (0-7)
///
/// # Returns
/// `true` if the bit at position `n` is set, `false` otherwise
#[inline]
pub fn bit(value: Byte, n: u8) -> bool {
    (value & (1 << n)) != 0
}

/// Set or clear a specific bit in a byte value
///
/// # Arguments
/// * `value` - Mutable reference to the byte value
/// * `n` - The bit position (0-7)
/// * `on` - `true` to set the bit, `false` to clear it
#[inline]
pub fn bit_set(value: &mut Byte, n: u8, on: bool) {
    if on {
        *value |= 1 << n;
    } else {
        *value &= !(1 << n);
    }
}

/// Check if a value is within a range (inclusive)
///
/// # Arguments
/// * `value` - The value to check
/// * `low` - The lower bound (inclusive)
/// * `high` - The upper bound (inclusive)
///
/// # Returns
/// `true` if `low <= value <= high`, `false` otherwise
#[inline]
pub fn between(value: Word, low: Word, high: Word) -> bool {
    value >= low && value <= high
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit() {
        assert!(bit(0b00000001, 0));
        assert!(!bit(0b00000001, 1));
        assert!(bit(0b10000000, 7));
        assert!(!bit(0b01111111, 7));
        assert!(bit(0b00010000, 4));
    }

    #[test]
    fn test_bit_set() {
        let mut value: Byte = 0;
        
        bit_set(&mut value, 0, true);
        assert_eq!(value, 0b00000001);
        
        bit_set(&mut value, 7, true);
        assert_eq!(value, 0b10000001);
        
        bit_set(&mut value, 0, false);
        assert_eq!(value, 0b10000000);
        
        bit_set(&mut value, 7, false);
        assert_eq!(value, 0);
    }

    #[test]
    fn test_between() {
        assert!(between(5, 0, 10));
        assert!(between(0, 0, 10));
        assert!(between(10, 0, 10));
        assert!(!between(11, 0, 10));
        assert!(!between(0xC000, 0x8000, 0x9FFF));
        assert!(between(0xC000, 0xC000, 0xDFFF));
    }
}
