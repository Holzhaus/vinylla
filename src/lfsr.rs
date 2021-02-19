//! Implementation of a Fibonacci Linear Feedback Shift Register (LFSR).
//!
//! An n-bit LFSR generates a bitstream from an n-bit state. For each cycle, the bits at certain
//! positions of the current state are XOR'ed and the result is fed back into the register. The
//! rightmost bit of the state that is "pushed out" of the register is the output bit.
//!
//! # Description
//!
//! *Note: Let a = n. We use a instead of n here because there is no subscript n in Unicode).*
//!
//! An LFSR can be described by the register's bit length (a) and the bit positions that influence
//! the next feedback bit. These bit positions are called "taps" and can be written as vector p =
//! (pₐ₋₁, ..., p₃, p₂, p₁, p₀) where each element can either be 0 or 1 (mathematically speaking: ∀
//! x ∈ ℕ: pₓ ∈ {0, 1}).
//!
//! That generic description may look complicated and daunting, but keep reading. There's and
//! example below that will make it clearer.
//!
//! ```text
//!      MSB                                    LSB
//!     ┌─────┐           ┌───┐  ┌───┐  ┌───┐  ┌───┐
//! ┌──▶│ sₐ₋₁├┬──▶ ... ─▶│ s₃├┬▶│ s₂├┬▶│ s₁├┬▶│ s₀├┬───▶ output bit
//! │   └─────┘│          └───┘│ └───┘│ └───┘│ └───┘│
//! │          ▼               ▼      ▼      ▼      ▼
//! │sₐ        ⊗ ◀─pₐ₋₁        ⊗ ◀─p₃ ⊗ ◀─p₂ ⊗ ◀─p₁ ⊗ ◀─p₀
//! │          │               │      │      │      │
//! │          ▼               ▼      ▼      ▼      │
//! └─────────╴⊕ ◀─ ... ◀──────⊕ ◀────⊕ ◀────⊕ ◀────┘
//! ```
//!
//! The LFSR state is a-bit vector s = (sₐ₋₁, ..., s₃, s₂, s₁, s₀).
//!
//! After the first clock cycle, the internal state is shifted, such that s = (sₐ, ..., s₄, s₃, s₂,
//! s₁) and s₀ becomes the output bit. The feedback bit can be calculated as:
//!
//! ```text
//! sₐ ≡ pₐ₋₁ × sₐ₋₁ + ... + p₃ × s₃ + p₂ × s₂ + p₁ × s₁ + p₁ × s₁ + p₀ × s₀ mod 2
//! ```
//!
//! It's important that the taps `p` are a property of the LFSR that doesn't change, so the
//! next feedback bit is calculated as:
//!
//! ```text
//! sₐ₊₁ ≡ pₐ₋₁ × sₐ + ... + p₃ × s₄ + p₂ × s₃ + p₁ × s₂ + p₁ × s₂ + p₀ × s₁ mod 2
//! ```
//!
//! # Example
//!
//! Let's consider an 8-bit LFSR with taps p = (0, 0, 0, 1, 1, 1, 0, 1). This information
//! suffices to draw it as:
//!
//! ```text
//!      MSB                                              LSB
//!     ┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐
//! ┌──▶│ s₇├─▶│ s₆├─▶│ s₅├─▶│ s₄├┬▶│ s₃├┬▶│ s₂├┬▶│ s₁├─▶│ s₀├┬───▶ output bit
//! │   └───┘  └───┘  └───┘  └───┘│ └───┘│ └───┘│ └───┘  └───┘│
//! │s₈                           │      │      │             │
//! │                             ▼      ▼      ▼             │
//! └─────────────────────────────⊕ ◀────⊕ ◀────⊕ ◀───────────┘
//! ```
//!
//! We can now calculate the output bit as:
//!
//! ```text
//! s₈ ≡ p₇ × s₇ + p₆ × s₆ + p₅ × s₅ + p₄ × s₄ + p₃ × s₃ + p₂ × s₂ + p₁ × s₁ + p₀ × s₀ mod 2
//!    ≡  0 × s₇ +  0 × s₆ +  0 × s₅ +  1 × s₄ +  1 × s₃ + 1  × s₂ +  0 × s₁ +  1 × s₀ mod 2
//!    ≡                                    s₄ +      s₃ +      s₂           +      s₀ mod 2
//!    ≡ s₄ ⊕ s₃ ⊕ s₂ ⊕  s₀
//! ```
//!
//! Let the initial state s = (1, 1, 0, 0, 1, 0, 0, 1).
//!
//! ```text
//!      MSB                                              LSB
//!       s₇     s₆     s₅     s₄     s₃     s₂     s₁     s₀
//!     ┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐
//! ┌──▶│ 1 ├─▶│ 1 ├─▶│ 0 ├─▶│ 0 ├┬▶│ 1 ├┬▶│ 0 ├┬▶│ 0 ├─▶│ 1 ├┬───▶ s₀
//! │   └───┘  └───┘  └───┘  └───┘│ └───┘│ └───┘│ └───┘  └───┘│
//! │s₈                           │      │      │             │
//! │                             ▼      ▼      ▼             │
//! └─────────────────────────────⊕ ◀────⊕ ◀────⊕ ◀───────────┘
//! ```
//!
//! The first clock cycle now shifts the register to the right using feedback bit s₈.
//! That bit can be calculated using the equation above:
//!
//! ```text
//! s₈ ≡ s₄ ⊕ s₃ ⊕ s₂ ⊕  s₀
//!    ≡ 0 ⊕ 1 ⊕ 0 ⊕  1
//!    ≡ 0
//! ```
//!
//! The output bit is the bit that gets "pushed out" of the register, i.e. s₀ = 0.
//!
//! After the first clock the LFSR has state s = (0, 1, 1, 0, 0, 1, 0, 0) and looks like this:
//!
//! ```text
//!      MSB                                              LSB
//!       s₈     s₇     s₆     s₅     s₄     s₃     s₂     s₁
//!     ┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐
//! ┌──▶│ 0 ├─▶│ 1 ├─▶│ 1 ├─▶│ 0 ├┬▶│ 0 ├┬▶│ 1 ├┬▶│ 0 ├─▶│ 0 ├┬───▶ s₁
//! │   └───┘  └───┘  └───┘  └───┘│ └───┘│ └───┘│ └───┘  └───┘│
//! │s₉                           │      │      │             │
//! │                             ▼      ▼      ▼             │
//! └─────────────────────────────⊕ ◀────⊕ ◀────⊕ ◀───────────┘
//! ```
//!
//! For the second clock cycle, the output bit is s₁ = 0 and we can calculate the feedback bit s₉ as:
//!
//! ```text
//! s₉ ≡ s₅ ⊕ s₄ ⊕ s₃ ⊕  s₁
//!    ≡ 0 ⊕ 0 ⊕ 1 ⊕  0
//!    ≡ 1
//! ```
//!
//! After the second clock the LFSR has state s = (1, 0, 1, 1, 0, 0, 1, 0) and looks like this:
//!
//! ```text
//!      MSB                                              LSB
//!       s₉     s₈     s₇     s₆     s₅     s₄     s₃     s₂
//!     ┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐  ┌───┐
//! ┌──▶│ 1 ├─▶│ 0 ├─▶│ 1 ├─▶│ 1 ├┬▶│ 0 ├┬▶│ 0 ├┬▶│ 1 ├─▶│ 0 ├┬───▶ s₂
//! │   └───┘  └───┘  └───┘  └───┘│ └───┘│ └───┘│ └───┘  └───┘│
//! │s₁₀                          │      │      │             │
//! │                             ▼      ▼      ▼             │
//! └─────────────────────────────⊕ ◀────⊕ ◀────⊕ ◀───────────┘
//! ```
//!
//! ## Feedback Polynomial
//!
//! Mathematicians love polynomials, so instead of using the size and the taps to describe an LFSR,
//! they often use a polynomial:
//!
//! ```text
//! P(x) = p₀ × xᵃ + p₁ × xᵃ⁻¹ + p₂ × xᵃ⁻² + ... + pₐ₋₁  × x¹ + x⁰
//! ```
//!
//! So for the 8-bit LFSR in the example, we had these taps:
//!
//! ```text
//! p = (p₇, p₆, p₅, p₄, p₃, p₂, p₁, p₀)
//!   = (0,  0,  0,  1,  1,  1,  0,  1)
//! ```
//!
//! Therefore, the feedback polynomial of that LFSR is:
//!
//! ```text
//! P(x) = p₀ × x⁸ + p₁ × x⁷ + p₂ × x⁶ + p₃ × x⁵ + p₄ × x⁴ + p₅ × x³ + p₆ × x² + p₇ × x¹ + x⁰
//!      =  1 × x⁸ +  0 × x⁷ +  1 × x⁶ +  1 × x⁵ +  1 × x⁴ +  0 × x³ +  0 × x² +  0 × x¹ + x⁰
//!      =      x⁸ +                x⁶ +      x⁵ +      x⁴ +                               x⁰
//!      = x⁸ + x⁶ + x⁵ + x⁴ + 1
//! ```
//!
use super::bits;

/// Fibonacci Linear Feedback Shift Register (LFSR)
pub struct FibonacciLfsr {
    pub size: usize,
    pub state: u32,
    pub taps: u32,
}

impl FibonacciLfsr {
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
    use super::FibonacciLfsr;

    fn find_lfsr_period(size: usize, seed: u32, taps: u32) -> Option<usize> {
        let mut lfsr = FibonacciLfsr {
            size,
            state: seed,
            taps,
        };
        let mut period: usize = 0;
        let last_state = lfsr.state;
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
        let mut lfsr = FibonacciLfsr {
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
