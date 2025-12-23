// Copyright (c) 2025 Jan Holthuis <holthuis.jan@gmail.com> et al.
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use crate::{
    bitstream::Bitstream, format::TimecodeFormat, pitch::PitchDetector,
    util::ExponentialWeightedMovingAverage,
};
use std::cmp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveCycleStatus {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimecodeDirection {
    Forwards,
    Backwards,
}

#[derive(Debug)]
pub struct TimecodeChannel {
    ewma: ExponentialWeightedMovingAverage,
    wave_cycle_status: WaveCycleStatus,
    peak_threshold: i32,
    crossed_zero: bool, // made this a permanent item in the struct instead of being allocated as temp value every sample
    sample: i32, // made this a permanent item in the struct instead of being allocated as temp value every sample
    threshold: i32, // made this a permanent item in the struct instead of being allocated as temp value every sample
}

const TIME_CONSTANT: f64 = 0.0001;

const fn sample_to_i32(sample: i16) -> i32 {
    (sample as i32) << 16
}

impl TimecodeChannel {
    const INITIAL_PEAK_THRESHOLD: i32 = 0;

    pub fn new(sample_rate_hz: f64) -> Self {
        let ewma = ExponentialWeightedMovingAverage::new(TIME_CONSTANT, sample_rate_hz);

        let wave_cycle_status = WaveCycleStatus::Positive;
        let peak_threshold = Self::INITIAL_PEAK_THRESHOLD;

        TimecodeChannel {
            ewma,
            wave_cycle_status,
            peak_threshold,
            crossed_zero: false,
            sample: 0,
            threshold: 0,
        }
    }

    /// Returns true if the wave has crossed zero.
    pub fn has_crossed_zero(&self, sample: i32) -> bool {
        match self.wave_cycle_status {
            WaveCycleStatus::Negative => sample > self.ewma.last_output,
            WaveCycleStatus::Positive => sample < self.ewma.last_output,
        }
    }

    /// Process a sample and detect zero crossing.
    pub fn process_sample(&mut self, sample: i32) -> bool {
        self.crossed_zero = self.has_crossed_zero(sample);
        if self.crossed_zero {
            self.wave_cycle_status = match self.wave_cycle_status {
                WaveCycleStatus::Negative => WaveCycleStatus::Positive,
                WaveCycleStatus::Positive => WaveCycleStatus::Negative,
            };
        }

        self.ewma.process(sample);

        self.crossed_zero
    }

    /// Reads a bit from the sample and adjust the threshold.
    pub fn bit_from_sample(&mut self, sample: i32) -> bool {
        self.sample = self.ewma.difference_to(sample).abs();
        self.peak_threshold = cmp::max(sample, self.peak_threshold);
        self.threshold = (f64::from(self.peak_threshold) * 0.9).trunc() as i32;
        self.sample > self.threshold
    }
}

#[derive(Debug)]
pub struct Timecode {
    bitstream: Bitstream,
    primary_channel: TimecodeChannel,
    secondary_channel: TimecodeChannel,
    direction: TimecodeDirection,
    pitch: PitchDetector,
    primary_crossed_zero: bool, // made this a permanent item in the struct instead of being allocated as temp value every sample
    secondary_crossed_zero: bool, // made this a permanent item in the struct instead of being allocated as temp value every sample
    primary_sample: i32, // made this a permanent item in the struct instead of being allocated as temp value every sample
    secondary_sample: i32, // made this a permanent item in the struct instead of being allocated as temp value every sample
    bit: bool, // made this a permanent item in the struct instead of being allocated as temp value every sample
}

impl Timecode {
    pub fn new(format: &TimecodeFormat, sample_rate_hz: f64) -> Self {
        let TimecodeFormat {
            size,
            seed,
            taps,
            signal_frequency_hz,
        } = format;

        let bitstream = Bitstream::new(*size, *seed, *taps);
        let primary_channel = TimecodeChannel::new(sample_rate_hz);
        let secondary_channel = TimecodeChannel::new(sample_rate_hz);

        let pitch = PitchDetector::new(sample_rate_hz, *signal_frequency_hz);

        Self {
            bitstream,
            primary_channel,
            secondary_channel,
            direction: TimecodeDirection::Forwards,
            pitch,
            primary_crossed_zero: false,
            secondary_crossed_zero: false,
            primary_sample: 0,
            secondary_sample: 0,
            bit: false,
        }
    }
    /// Returns the current state of the bitstream
    pub fn state(&self) -> u32 {
        self.bitstream.state()
    }

    pub fn set_state(&mut self, state: u32) {
        self.bitstream.set_state(state);
    }

    pub fn process_channels(
        &mut self,
        primary_sample: i16,
        secondary_sample: i16,
    ) -> Option<(bool, Option<u32>)> {
        self.primary_sample = sample_to_i32(primary_sample);
        self.secondary_sample = sample_to_i32(secondary_sample);
        self.primary_crossed_zero = self.primary_channel.process_sample(self.primary_sample);
        self.secondary_crossed_zero = self.secondary_channel.process_sample(self.secondary_sample);

        // Detect the playback direction of the timecode.
        //
        //                         Assuming the primary channel crossed zero:
        //  ──╮   ╭───╮   ╭(4)╮    If both the primary wave and the secondary
        //    │  (2)  │   │   │    wave are negative (1) or both are positive
        //  ─────────────────────  (2), then the timecode is playing forwards,
        //   (1)  │   │   │   │    otherwise it's playing backwards.
        //    ╰───╯   ╰(3)╯   ╰──
        //                         Assuming the secondary channel crossed zero:
        //  ╮   ╭(2)╮   ╭───╮   ╭  If the primary wave is negative and the
        //  │   │   │  (3)  │   │  secondary wave is positive (3) or if the
        //  ─────────────────────  primary wave is positive and the secondary
        //  │   │   │   │  (4)  │  wave is positive (4), the timecode is playing
        //  ╰(1)╯   ╰───╯   ╰───╯  forwards, otherwise it's playing backwards.
        //
        if self.primary_crossed_zero {
            self.direction = if self.primary_channel.wave_cycle_status
                == self.secondary_channel.wave_cycle_status
            {
                TimecodeDirection::Forwards
            } else {
                TimecodeDirection::Backwards
            }
        } else if self.secondary_crossed_zero {
            self.direction = if self.primary_channel.wave_cycle_status
                != self.secondary_channel.wave_cycle_status
            {
                TimecodeDirection::Forwards
            } else {
                TimecodeDirection::Backwards
            }
        }

        // The timecode has a frequency of 1000 Hz and the sample rate is 44100 Hz.
        // This means a cycle at full playback rate takes 44.1 samples to complete.
        //
        //     ⇤ Cycle ⇥
        //  ──╮┋  ╭───╮┋  ╭
        //    │┋  │   │┋  │
        //  ───┋──2───4┋───
        //    │┋  │   │┋  │
        //    ╰┋──╯   ╰┋──╯
        //     ┋       ┋
        //  ╮  ┋╭───╮  ┋╭──
        //  │  ┋│   │  ┋│
        //  ───┋1───3──┋───
        //  │  ┋│   │  ┋│
        //  ╰──┋╯   ╰──┋╯
        //
        // For each cycle, the wave for each channel crosses zero 2 times, so there are 4 zero
        // crossings per cycle total. This means there should be 4 zero crossings per 44.1 samples
        // if the record is playing with full speed. With the record is played back with double
        // speed, it takes 22.05 samples to complete a cycle (in other words: to detect 4 zero
        // crossings), and if it plays with half speed, it takes 88.2 samples.
        //
        // This means we can count the number of samples of the last current cycle, and then
        // calculate the pitch as 44.1 / number_of_samples_of_this_cycle.
        //
        // To get faster responses, we can simply count the number of samples per quarter cycle
        // (i.e. per single zero crossing) then calculate:
        // pitch = 11.025 / number_of_samples_since_previous_zero_crossing
        if self.primary_crossed_zero || self.secondary_crossed_zero {
            self.pitch.update_after_zero_crossing(
                self.primary_sample,
                self.secondary_sample,
                self.primary_crossed_zero,
            );
        } else {
            self.pitch
                .update(self.primary_sample, self.secondary_sample);
        }

        // Read a bit from the timecode.
        //
        // The timecode waveform has a constant frequency with a variable
        // amplitude. The variations in the amplitude encode the binary data
        // stream. The primary channel's amplitude is read as a bit when
        // the secondary channel's waveform crosses 0 and the primary
        // channel's waveform is positive. Peaks with a larger amplitude
        // are bit 1 (diagram positions 1 and 3) and peaks with a lower
        // amplitude are bit 0 (diagram position 2).
        //
        //    "1"             "1"
        //   ╭───╮    "0"    ╭───╮
        //   │   │   ╭───╮   │   │
        // ───(1)─────(2)─────(3)───  primary channel
        //   │   ╰───╯   │   │   │
        // ──╯           ╰───╯   ╰──
        //
        // ╭───╮           ╭───╮   ╭
        // │   │   ╭───╮   │   │   │
        // ───(1)─────(2)─────(3)───  secondary channel
        // │   ╰───╯   │   │   │   │
        // ╯           ╰───╯   ╰───╯
        //
        if self.secondary_crossed_zero
            && self.primary_channel.wave_cycle_status == WaveCycleStatus::Positive
        {
            self.bit = self.primary_channel.bit_from_sample(self.primary_sample);
            if self.direction == TimecodeDirection::Forwards {
                self.bitstream.process_bit(self.bit as u32);
            } else {
                self.bitstream.process_bit_backward(self.bit as u32);
            }
            return Some((self.bit, self.bitstream.position()));
        }

        None
    }
}
