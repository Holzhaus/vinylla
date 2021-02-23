// vinylla - (c) 2021 Jan Holthuis <holthuis.jan@gmail.com> et al.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#[derive(Debug, Clone, PartialEq)]
pub struct TimecodeFormat {
    pub size: usize,
    pub seed: u32,
    pub taps: u32,
    pub signal_frequency_hz: f64,
}

/// Serato Control CD 1.0.0
///
/// The Serato Control CD can be downloaded free of cost [from the Serato
/// Website](https://serato.com/controlcd/downloads) as zipped WAV file or ISO image.
pub const SERATO_CONTROL_CD_1_0_0: TimecodeFormat = TimecodeFormat {
    size: 20,
    seed: 0b1001_0001_0100_1010_1011,
    /// LFSR feedback polynomial:
    /// x^20 + x^18 + x^16 + x^14 + x^12 + x^10 + x^9 + x^6 + x^4 + x^3 + 1
    taps: 0b0011_0100_1101_0101_0101,
    signal_frequency_hz: 1000.0,
};
