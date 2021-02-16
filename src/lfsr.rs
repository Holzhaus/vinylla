//! Implementation of a Fibonacci Linear Feedback Shift Register (LFSR).
//!
//! An n-bit LFSR generates a bitstream from an n-bit state. For each cycle, the bits at certain
//! positions of the current state are XOR'ed and the result is fed back into the register. The
//! rightmost bit of the state that is "pushed out" of the register is the output bit.
//!
//! ```text
//!       MSB                         LSB
//!      ┌───┬───┬───┬───┬───┬───┬───┬───┐
//! ┌──▶ │ a₇│ a₆│ a₅│ a₄│ a₃│ a₂│ a₁│ a₀│ ───▶ b (output bit)
//! │    └───┴───┴───┴─┬─┴─┬─┴─┬─┴───┴─┬─┘
//! │                  │   │   │       │
//! │                  ▼   ▼   ▼       │
//! └─────────────────╴⊕ ◀╴⊕ ◀╴⊕ ◀─────┘
//!     c (feedback bit)
//! ```
//!
//! The example above depicts an 8-bit LFSR with the feedback polynomial x⁸ + x⁶ + x⁵ + x⁴ + 1.
//!
//! When writing the LFSR state as vector of bits a = (a₇, a₆, a₅, a₄, a₃, a₂, a₁, a₀), the binary
//! `taps` represenation of that LFSR is 00011101₂, i.e. every bit that influences the input is 1,
//! all other bits are 0.
//!
//! The feedback bit can be calculated as:
//! c = 0⋅a₇ + 0⋅a₆ + 0⋅a₅ + 1⋅a₄ + 1⋅a₃ + 1⋅a₂ + 0⋅a₁ + 1⋅a₀ mod 2 = a₀ ⊕ a₂⊕ a₃ ⊕ a₄
use super::bits;

/// Fibonacci Linear Feedback Shift Register (LFSR)
pub struct LFSR {
    pub size: usize,
    pub state: u32,
    pub taps: u32,
}

impl LFSR {
    /// Return the next LFSR state (without making any changes).
    pub const fn next_state(&self) -> u32 {
        let next_bit = (self.state & self.taps).count_ones() & 1;
        bits::insert_msb(self.size, self.state, next_bit)
    }

    /// Return the previous LFSR state (without making any changes).
    pub const fn previous_state(&self) -> u32 {
        let taps = bits::rotate_right(self.size, self.taps);
        let previous_bit = (self.state & taps).count_ones() & 1;
        bits::insert_lsb(self.size, self.state, previous_bit)
    }

    /// Advance the LFSR state and return it.
    pub fn advance(&mut self) -> u32 {
        self.state = self.next_state();
        self.state
    }

    /// Revert the LFSR state and return it.
    pub fn revert(&mut self) -> u32 {
        self.state = self.previous_state();
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::LFSR;

    fn find_lfsr_period(size: usize, seed: u32, taps: u32) -> Option<usize> {
        let mut lfsr = LFSR {
            size,
            state: seed,
            taps,
        };
        let mut period: usize = 0;
        let mut last_state = lfsr.state;
        while period < usize::MAX {
            let state = lfsr.advance();
            period += 1;
            if lfsr.state == seed {
                return Some(period);
            }
            assert_eq!(lfsr.state, state);
            assert_ne!(lfsr.state, last_state);
        }
        None
    }

    #[test]
    fn test_maximal_length_lfsrs() {
        // Test a bunch of maximum length LFSRs (i. e. b-bit LFSRs that generate an bitstream with
        // a 2^n - 1 period).
        let configurations = [
            (2, 0b11),            // x^2 + x + 1
            (3, 0b011),           // x^3 + x^2 + 1
            (4, 0b0011),          // x^4 + x^3 + 1
            (5, 0b00101),         // x^5 + x^3 + 1
            (6, 0b000011),        // x^6 + x^5 + 1
            (7, 0b0000011),       // x^7 + x^6 + 1
            (8, 0b00011101),      // x^8 + x^6 + x^5 + x^4 + 1
            (9, 0b000010001),     // x^9 + x^5 + 1
            (10, 0b0000001001),   // x^10 + x^7 + 1
            (11, 0b00000000101),  // x^11 + x^9 + 1
            (12, 0b000100000111), // x^12 + x^11 + x^10 + x^4 + 1
        ];

        let seed = 1;
        for &(size, taps) in configurations.iter() {
            let expected_period = 2_usize.pow(size as u32) - 1;
            let actual_period =
                find_lfsr_period(size, seed, taps).expect("Failed to find LFSR period!");
            assert_eq!(expected_period, actual_period, "Unexpected LFSR period!");
        }
    }

    #[test]
    fn test_lfsr_advance_and_revert() {
        let mut lfsr = LFSR {
            state: 0b10101,
            size: 5,
            taps: 0b00101,
        };
        assert_eq!(lfsr.state, 0b10101);

        assert_eq!(lfsr.advance(), 0b01010);
        assert_eq!(lfsr.advance(), 0b00101);
        assert_eq!(lfsr.advance(), 0b00010);
        assert_eq!(lfsr.advance(), 0b00001);
        assert_eq!(lfsr.advance(), 0b10000);
        assert_eq!(lfsr.advance(), 0b01000);
        assert_eq!(lfsr.advance(), 0b00100);
        assert_eq!(lfsr.advance(), 0b10010);
        assert_eq!(lfsr.advance(), 0b01001);
        assert_eq!(lfsr.advance(), 0b10100);
        assert_eq!(lfsr.advance(), 0b11010);
        assert_eq!(lfsr.advance(), 0b01101);
        assert_eq!(lfsr.advance(), 0b00110);
        assert_eq!(lfsr.advance(), 0b10011);
        assert_eq!(lfsr.advance(), 0b11001);
        assert_eq!(lfsr.advance(), 0b11100);
        assert_eq!(lfsr.advance(), 0b11110);
        assert_eq!(lfsr.advance(), 0b11111);
        assert_eq!(lfsr.advance(), 0b01111);
        assert_eq!(lfsr.advance(), 0b00111);
        assert_eq!(lfsr.advance(), 0b00011);
        assert_eq!(lfsr.advance(), 0b10001);
        assert_eq!(lfsr.advance(), 0b11000);
        assert_eq!(lfsr.advance(), 0b01100);
        assert_eq!(lfsr.advance(), 0b10110);
        assert_eq!(lfsr.advance(), 0b11011);
        assert_eq!(lfsr.advance(), 0b11101);
        assert_eq!(lfsr.advance(), 0b01110);
        assert_eq!(lfsr.advance(), 0b10111);
        assert_eq!(lfsr.advance(), 0b01011);
        assert_eq!(lfsr.advance(), 0b10101);

        assert_eq!(lfsr.revert(), 0b01011);
        assert_eq!(lfsr.revert(), 0b10111);
        assert_eq!(lfsr.revert(), 0b01110);
        assert_eq!(lfsr.revert(), 0b11101);
        assert_eq!(lfsr.revert(), 0b11011);
        assert_eq!(lfsr.revert(), 0b10110);
        assert_eq!(lfsr.revert(), 0b01100);
        assert_eq!(lfsr.revert(), 0b11000);
        assert_eq!(lfsr.revert(), 0b10001);
        assert_eq!(lfsr.revert(), 0b00011);
        assert_eq!(lfsr.revert(), 0b00111);
        assert_eq!(lfsr.revert(), 0b01111);
        assert_eq!(lfsr.revert(), 0b11111);
        assert_eq!(lfsr.revert(), 0b11110);
        assert_eq!(lfsr.revert(), 0b11100);
        assert_eq!(lfsr.revert(), 0b11001);
        assert_eq!(lfsr.revert(), 0b10011);
        assert_eq!(lfsr.revert(), 0b00110);
        assert_eq!(lfsr.revert(), 0b01101);
        assert_eq!(lfsr.revert(), 0b11010);
        assert_eq!(lfsr.revert(), 0b10100);
        assert_eq!(lfsr.revert(), 0b01001);
        assert_eq!(lfsr.revert(), 0b10010);
        assert_eq!(lfsr.revert(), 0b00100);
        assert_eq!(lfsr.revert(), 0b01000);
        assert_eq!(lfsr.revert(), 0b10000);
        assert_eq!(lfsr.revert(), 0b00001);
        assert_eq!(lfsr.revert(), 0b00010);
        assert_eq!(lfsr.revert(), 0b00101);
        assert_eq!(lfsr.revert(), 0b01010);
    }
}
