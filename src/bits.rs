// vinylla - (c) 2021 Jan Holthuis <holthuis.jan@gmail.com> et al.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Low level bitwise operations

/// Return 2^size - 1 that can be used as a bitmask
pub const fn mask(size: usize) -> u32 {
    (1 << size) - 1
}

/// Shift all bits in `size`-bit integer `data` to the right and set `bit` as MSB.
///
/// The LSB of `data` before the shift will be discarded.
pub const fn insert_msb(size: usize, data: u32, bit: u32) -> u32 {
    let bit = bit & 1;
    (bit << (size - 1)) | (data >> 1)
}

/// Shift all bits in `size`-bit integer `data` to the left and set `bit` as LSB.
///
/// The MSB of `data` before the shift will be discarded.
pub const fn insert_lsb(size: usize, data: u32, bit: u32) -> u32 {
    let bit = bit & 1;
    (data << 1) & mask(size) | bit
}

/// Shift all bits in `size`-bit integer `data` to the left and use the old MSB as new LSB.
pub const fn rotate_left(size: usize, data: u32) -> u32 {
    let msb = data >> (size - 1);
    insert_lsb(size, data, msb)
}

/// Shift all bits in `size`-bit integer `data` to the right and use the old LSB as new MSB.
pub const fn rotate_right(size: usize, data: u32) -> u32 {
    let lsb = data & 1;
    insert_msb(size, data, lsb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask() {
        assert_eq!(mask(0), 0b00000000);
        assert_eq!(mask(1), 0b00000001);
        assert_eq!(mask(5), 0b00011111);
        assert_eq!(mask(8), 0b11111111);
    }

    #[test]
    fn test_insert_msb() {
        assert_eq!(insert_msb(5, 0b10101, 0), 0b01010);
        assert_eq!(insert_msb(5, 0b10101, 1), 0b11010);

        assert_eq!(insert_msb(4, 0b0101, 0), 0b0010);
        assert_eq!(insert_msb(4, 0b0101, 1), 0b1010);

        assert_eq!(insert_msb(16, 0b1111000011110000, 0), 0b0111100001111000);
        assert_eq!(insert_msb(16, 0b1111000011110000, 1), 0b1111100001111000);
    }

    #[test]
    fn test_insert_lsb() {
        assert_eq!(insert_lsb(5, 0b10101, 0), 0b01010);
        assert_eq!(insert_lsb(5, 0b10101, 1), 0b01011);

        assert_eq!(insert_lsb(4, 0b0101, 0), 0b1010);
        assert_eq!(insert_lsb(4, 0b0101, 1), 0b1011);

        assert_eq!(insert_lsb(16, 0b1111000011110000, 0), 0b1110000111100000);
        assert_eq!(insert_lsb(16, 0b1111000011110000, 1), 0b1110000111100001);
    }

    #[test]
    fn test_rotate_left() {
        assert_eq!(rotate_left(5, 0b10101), 0b01011);
        assert_eq!(rotate_left(5, 0b01011), 0b10110);

        assert_eq!(rotate_left(4, 0b1101), 0b1011);
        assert_eq!(rotate_left(4, 0b1011), 0b0111);

        assert_eq!(rotate_left(16, 0b1111000011110000), 0b1110000111100001);
        assert_eq!(rotate_left(16, 0b1110000111100001), 0b1100001111000011);
    }

    #[test]
    fn test_rotate_right() {
        assert_eq!(rotate_right(5, 0b10101), 0b11010);
        assert_eq!(rotate_right(5, 0b11010), 0b01101);

        assert_eq!(rotate_right(4, 0b0111), 0b1011);
        assert_eq!(rotate_right(4, 0b1011), 0b1101);

        assert_eq!(rotate_right(16, 0b1111000011110000), 0b0111100001111000);
        assert_eq!(rotate_right(16, 0b0111100001111000), 0b0011110000111100);
    }
}
