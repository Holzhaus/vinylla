// Copyright (c) 2025 Jan Holthuis <holthuis.jan@gmail.com> et al.
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

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
    // LFSR feedback polynomial:
    // x^20 + x^18 + x^16 + x^14 + x^12 + x^10 + x^9 + x^6 + x^4 + x^3 + 1
    taps: 0b0011_0100_1101_0101_0101,
    signal_frequency_hz: 1000.0,
};

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Timecode, TimecodeAudioGenerator};

    fn test_format(format: &TimecodeFormat, sample_rate_hz: f64) {
        let mut generator = TimecodeAudioGenerator::new(format, sample_rate_hz);
        let mut timecode = Timecode::new(format, sample_rate_hz);
        let initial_state = generator.state();
        let mut previous_timecode_state = timecode.state();
        let mut state_changed = false;
        assert_eq!(timecode.state(), initial_state);
        assert_eq!(timecode.state(), generator.state());
        println!(
            "{:0>size$b} {:0>size$b}",
            timecode.state(),
            generator.state(),
            size = format.size,
        );

        // Skip the first few samples until the bit detection works properly
        for _ in 0..20 {
            let (left, right) = generator.next_sample();
            timecode.process_channels(left, right);
        }
        timecode.set_state(generator.state());

        loop {
            let (left, right) = generator.next_sample();
            timecode.process_channels(left, right);
            if timecode.state() != previous_timecode_state {
                println!(
                    "{:0>size$b} {:0>size$b}",
                    timecode.state(),
                    generator.state(),
                    size = format.size,
                );

                assert_eq!(timecode.state(), generator.state());
                previous_timecode_state = timecode.state();
                state_changed = true;
            }

            if state_changed && generator.state() == initial_state {
                break;
            }
        }
    }

    #[test]
    fn test_serato_control_cd_1_0_0_44100hz() {
        test_format(&SERATO_CONTROL_CD_1_0_0, 44100.0);
    }

    #[test]
    fn test_serato_control_cd_1_0_0_48000hz() {
        test_format(&SERATO_CONTROL_CD_1_0_0, 48000.0);
    }
}
