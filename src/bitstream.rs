//! The [`Bitstream` struct](Bitstream) processes bits and maps them to positions.

use crate::{bits, lfsr::FibonacciLfsr};
use std::collections::HashMap;

/// Maps a bitstream to a position in the underlying lookup table.
///
/// The [`Bitstream` struct](Bitstream) uses an n-bit LFSR to populate a lookup table (LUT), which
/// can then be used to retrieve a position for some n-bit sequence.
#[derive(Debug)]
pub struct Bitstream {
    lookup_table: HashMap<u32, u32>,
    size: usize,
    bitstream: u32,
    valid_bits: usize,
}

impl Bitstream {
    /// Create a timecode bitstream using a LFSR with length `capacity`.
    pub fn new(size: usize, seed: u32, taps: u32) -> Self {
        // Precompute lookup table
        let capacity = 2u32.pow(size as u32) - 1;
        let mut lfsr = FibonacciLfsr {
            size,
            state: seed,
            taps,
        };
        let mut lookup_table = HashMap::with_capacity(capacity as usize);
        for i in 0..capacity {
            lookup_table.insert(lfsr.state, i);
            lfsr.advance();
        }

        Self {
            lookup_table,
            size,
            bitstream: seed,
            valid_bits: size,
        }
    }

    /// Process a single bit in forwards direction.
    ///
    /// If the positions before and after inserting the bit are not consecutive, the bitstream
    /// is marked as invalid. Processing more bits will let the bitstream become valid again.
    pub fn process_bit(&mut self, bit: u32) {
        let prev_position = self.position();
        self.bitstream = bits::insert_msb(self.size, self.bitstream, bit);
        if let Some(prev_position) = prev_position {
            let next_position = self.position();
            if let Some(next_position) = next_position {
                if prev_position + 1 != next_position {
                    // Discard all previously processed bits
                    self.valid_bits = 0
                }
            }
        }
        self.valid_bits += 1;
    }

    /// Process a single bit in backwards direction.
    ///
    /// If the positions before and after inserting the bit are not consecutive, the bitstream
    /// is marked as invalid. Processing more bits will let the bitstream become valid again.
    pub fn process_bit_backward(&mut self, bit: u32) {
        let prev_position = self.position();
        self.bitstream = bits::insert_lsb(self.size, self.bitstream, bit);
        if let Some(prev_position) = prev_position {
            let next_position = self.position();
            if let Some(next_position) = next_position {
                if prev_position != next_position + 1 {
                    // Discard all previously processed bits
                    self.valid_bits = 0;
                }
            }
        }
        self.valid_bits += 1;
    }

    /// Returns `true` if the position is considered valid.
    pub fn is_valid(&self) -> bool {
        self.valid_bits >= self.size
    }

    /// Retrieve the Position from the current bitstream.
    ///
    /// Returns None if the bitstream is considered invalid.
    pub fn position(&self) -> Option<u32> {
        if !self.is_valid() {
            return None;
        }

        self.lookup_table
            .get(&self.bitstream)
            .map(ToOwned::to_owned)
    }
}

#[cfg(test)]
mod test {
    use super::Bitstream;

    #[test]
    fn test_lookup_table() {
        let mut timecode = Bitstream::new(8, 0b00000001, 0b00011101);
        assert_eq!(timecode.position(), Some(0));
        assert_eq!(timecode.valid_bits, 8);

        // old state:  0b00000001
        // taps:       0b00011101
        // next input: 0b00000001.count_ones() mod 2 = 1
        timecode.process_bit(1);
        // new state:  0b10000000

        assert_eq!(timecode.position(), Some(1));
        assert_eq!(timecode.valid_bits, 9);

        // old state:  0b10000000
        // taps:       0b00011101
        // next input: 0b00000000.count_ones() mod 2 = 0
        timecode.process_bit(0);
        // new state:  0b01000000

        assert_eq!(timecode.position(), Some(2));
        assert_eq!(timecode.valid_bits, 10);

        // old state:  0b01000000
        // taps:       0b00011101
        // next input: 0b00000000.count_ones() mod 2 = 0
        timecode.process_bit(0);
        // new state:  0b00100000

        assert_eq!(timecode.position(), Some(3));
        assert_eq!(timecode.valid_bits, 11);

        // old state:  0b00100000
        // taps:       0b00011101
        // next input: 0b00000000.count_ones() mod 2 = 0
        timecode.process_bit(0);
        // new state:  0b00011000

        assert_eq!(timecode.position(), Some(4));
        assert_eq!(timecode.valid_bits, 12);

        // old state:  0b00010000
        // taps:       0b00011101
        // next input: 0b00010000.count_ones() mod 2 = 1
        //
        // Here, we simulate skipping, resulting in an invalid bitstream until at least 8 bits were
        // processed. Hence, we push 0 even though the next bit is expected to be 0.
        timecode.process_bit(0);
        assert_eq!(timecode.position(), None);
        assert_eq!(timecode.valid_bits, 1);

        timecode.process_bit(0);
        assert_eq!(timecode.position(), None);
        assert_eq!(timecode.valid_bits, 2);

        timecode.process_bit(1);
        assert_eq!(timecode.position(), None);
        assert_eq!(timecode.valid_bits, 3);

        timecode.process_bit(1);
        assert_eq!(timecode.position(), None);
        assert_eq!(timecode.valid_bits, 4);

        timecode.process_bit(0);
        assert_eq!(timecode.position(), None);
        assert_eq!(timecode.valid_bits, 5);

        timecode.process_bit(0);
        assert_eq!(timecode.position(), None);
        assert_eq!(timecode.valid_bits, 6);

        timecode.process_bit(1);
        assert_eq!(timecode.position(), None);
        assert_eq!(timecode.valid_bits, 7);

        timecode.process_bit(1);

        // At this point, 8 consecutive bits were processed, so bitstream is valid again
        assert_eq!(timecode.bitstream, 0b11001100);
        assert_eq!(timecode.position(), Some(182));
        assert_eq!(timecode.valid_bits, 8);

        // old state:  0b11001100
        // taps:       0b00011101
        // next input: 0b00001100.count_ones() mod 2 = 0
        timecode.process_bit(0);
        // new state:  0b01100110

        assert_eq!(timecode.position(), Some(183));
        assert_eq!(timecode.valid_bits, 9);

        // old state:  0b01100110
        // taps:       0b00011101
        // next input: 0b00000100.count_ones() mod 2 = 1
        timecode.process_bit(1);
        // new state:  0b10110011

        assert_eq!(timecode.position(), Some(184));
        assert_eq!(timecode.valid_bits, 10);

        // old state:  0b10110011
        // taps:       0b00011101
        // next input: 0b00010001.count_ones() mod 2 = 0
        timecode.process_bit(0);
        // new state:  0b01011001

        assert_eq!(timecode.position(), Some(185));
        assert_eq!(timecode.valid_bits, 11);

        timecode.process_bit_backward(1);
        assert_eq!(timecode.position(), Some(184));
        assert_eq!(timecode.valid_bits, 12);

        timecode.process_bit_backward(0);
        assert_eq!(timecode.position(), Some(183));
        assert_eq!(timecode.valid_bits, 13);

        timecode.process_bit(1);
        assert_eq!(timecode.position(), Some(184));
        assert_eq!(timecode.valid_bits, 14);
    }

    #[test]
    fn test_exactly_1_bit_produces_consecutive_positions() {
        // At any point in time, you can either process a 1 or a 0.
        //
        // Let a be the position before processing bit x and b be the position after processing
        // bit x. Then a and b should be consecutive positions for exactly one x ∈ {0, 1}.

        // Process bit 0 and check if positions are consecutive
        let mut timecode0 = Bitstream::new(8, 0b11110000, 0b00011101);
        let position0_a = timecode0.position();
        timecode0.process_bit(0);
        let position0_b = timecode0.position();
        let consecutive0 = if let (Some(a), Some(b)) = (position0_a, position0_b) {
            a + 1 == b
        } else {
            false
        };

        // Now do the same for bit 1
        let mut timecode1 = Bitstream::new(8, 0b11110000, 0b00011101);
        let position1_a = timecode1.position();
        timecode1.process_bit(1);
        let position1_b = timecode1.position();
        let consecutive1 = if let (Some(a), Some(b)) = (position1_a, position1_b) {
            a + 1 == b
        } else {
            false
        };

        assert_ne!(consecutive0, consecutive1);
    }
}
