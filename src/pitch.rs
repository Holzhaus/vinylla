// vinylla - (c) 2021 Jan Holthuis <holthuis.jan@gmail.com> et al.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#[derive(Debug, Clone, Copy)]
pub struct PitchDetector {
    samples_per_quarter_cycle: f64,
    samples_since_last_quarter_cycle: f64,
    last_primary_sample: i32,
    last_secondary_sample: i32,
}

impl PitchDetector {
    pub fn new(sample_rate_hz: f64, timecode_frequency_hz: f64) -> Self {
        let samples_per_quarter_cycle = sample_rate_hz / timecode_frequency_hz / 4.0;

        PitchDetector {
            samples_per_quarter_cycle,
            samples_since_last_quarter_cycle: 1.0,
            last_primary_sample: 0,
            last_secondary_sample: 0,
        }
    }

    pub fn update(&mut self, primary_sample: i32, secondary_sample: i32) {
        self.last_primary_sample = primary_sample;
        self.last_secondary_sample = secondary_sample;
        self.samples_since_last_quarter_cycle += 1.0;
    }

    pub fn update_after_zero_crossing(
        &mut self,
        primary_sample: i32,
        secondary_sample: i32,
        primary_crossed_zero: bool,
    ) -> f64 {
        // If a channel crossed zero, we now know the last sample value a (before the zero
        // crossing) and the current sample value b (after the crossing).
        //
        //  a
        //   \
        //  ─────
        //     \
        //      b
        //
        // We could now assume that the zero crossing happened just now, but it's more precise to
        // interpolate the exact subsample position where zero was crossed. This can be done by
        // assuming that the zero crossing position is equal to the proportion how far both are
        // from zero, e.g.:
        //
        // samples_since_zero_crossing = |b|/(|b| + |a|)
        //
        // This gives a number between 0.0 (if b is almost 0, i.e. the zero crossing is close to b)
        // and 1.0 (if a is almost 0, i.e. the zero crossing was immediately after sampling a).
        let samples_since_zero_crossing = if primary_crossed_zero {
            let b = f64::from(primary_sample.abs());
            b / (b + f64::from(self.last_primary_sample.abs()))
        } else {
            let b = f64::from(secondary_sample.abs());
            b / (b + f64::from(self.last_secondary_sample.abs()))
        };

        let samples_since_last_quarter_cycle =
            self.samples_since_last_quarter_cycle + 1.0 - samples_since_zero_crossing;

        let pitch = self.samples_per_quarter_cycle / samples_since_last_quarter_cycle;
        self.samples_since_last_quarter_cycle = samples_since_zero_crossing;
        pitch
    }
}
