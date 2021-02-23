//! Timecode takes a stream of stereo PCM audio from a timecode vinyl record or CD and decodes
//! the playback direction, speed, and position.
//!
//! # Direction Detection
//!
//! The timecode audio signal is a constant frequency.
//! The stereo channels carry the same signal out of phase by a quarter waveform period.
//! The direction is detected by comparing the polarity of the channels after either of them
//! cross 0:
//!
//! ```text
//!                         Assuming the primary channel crossed zero:
//!  ──╮   ╭───╮   ╭(4)╮    If both the primary wave and the secondary
//!    │  (2)  │   │   │    wave are negative (1) or both are positive
//!  ─────────────────────  (2), then the timecode is playing forwards,
//!   (1)  │   │   │   │    otherwise it's playing backwards.
//!    ╰───╯   ╰(3)╯   ╰──
//!                         Assuming the secondary channel crossed zero:
//!  ╮   ╭(2)╮   ╭───╮   ╭  If the primary wave is negative and the
//!  │   │   │  (3)  │   │  secondary wave is positive (3) or if the
//!  ─────────────────────  primary wave is positive and the secondary
//!  │   │   │   │  (4)  │  wave is positive (4), the timecode is playing
//!  ╰(1)╯   ╰───╯   ╰───╯  forwards, otherwise it's playing backwards.
//! ```
//!
//! # Speed Detection
//!
//! The timecode has a frequency of 1000 Hz. With a sample rate of 44100 Hz,
//! a cycle at full playback rate takes 44.1 samples to complete.
//!
//! ```text
//!     ⇤ Cycle ⇥
//!  ──╮┋  ╭───╮┋  ╭
//!    │┋  │   │┋  │
//!  ───┋──2───4┋───
//!    │┋  │   │┋  │
//!    ╰┋──╯   ╰┋──╯
//!     ┋       ┋
//!  ╮  ┋╭───╮  ┋╭──
//!  │  ┋│   │  ┋│
//!  ───┋1───3──┋───
//!  │  ┋│   │  ┋│
//!  ╰──┋╯   ╰──┋╯
//! ```
//!
//! For each cycle, the wave for each channel crosses zero 2 times, so there are 4 zero
//! crossings per cycle total. This means there should be 4 zero crossings per 44.1 samples
//! if the record is playing with full speed. With the record is played back with double
//! speed, it takes 22.05 samples to complete a cycle (in other words: to detect 4 zero
//! crossings), and if it plays with half speed, it takes 88.2 samples.
//!
//! This means [PitchDetector](crate::pitch) can count the number of samples of the last
//! current cycle, and then calculate the pitch as 44.1 / number_of_samples_of_this_cycle.
//!
//! To get faster responses, PitchDetector can simply count the number of samples per quarter
//! cycle (i.e. per single zero crossing) then calculate:
//! pitch = 11.025 / number_of_samples_since_previous_zero_crossing
//!
//! # Position Detection
//!
//! While the frequency is constant, the amplitude varies. The variations in amplitude encode
//! a binary data stream. The primary channel's amplitude is read as a bit when the secondary
//! channel's waveform crosses 0 and the primary channel's waveform is positive. Peaks with a
//! larger amplitude are bit 1 (diagram positions 1 and 3) and peaks with a lower amplitude are
//! bit 0 (diagram position 2).
//!
//! ```text
//!    "1"             "1"
//!   ╭───╮    "0"    ╭───╮
//!   │   │   ╭───╮   │   │
//! ───(1)─────(2)─────(3)───  primary channel
//!   │   ╰───╯   │   │   │
//! ──╯           ╰───╯   ╰──
//!
//! ╭───╮           ╭───╮   ╭
//! │   │   ╭───╮   │   │   │
//! ───(1)─────(2)─────(3)───  secondary channel
//! │   ╰───╯   │   │   │   │
//! ╯           ╰───╯   ╰───╯
//!
//! ```
//!
//! The binary [bitstream](crate::bitstream) is the output of an [LFSR](crate::lfsr), allowing
//! a short sequence of bits anywhere in the bitstream to identify a unique position without
//! a need for word boundaries.
use crate::{
    bitstream::Bitstream, format::TimecodeFormat, pitch::PitchDetector,
    util::ExponentialWeightedMovingAverage,
};

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
    samples_since_zero_crossing: usize,
    peak_threshold: i32,
}

const SAMPLE_RATE_HZ: f64 = 44100.0;
const TIMECODE_FREQUENCY_HZ: f64 = 1000.0;
const TIME_CONSTANT: f64 = 0.001;

const fn sample_to_i32(sample: i16) -> i32 {
    (sample as i32) << 16
}

impl TimecodeChannel {
    const ZERO_CROSSING_THRESHOLD: i32 = sample_to_i32(128);
    const INITIAL_PEAK_THRESHOLD: i32 = i32::MAX;

    pub fn new() -> Self {
        let ewma = ExponentialWeightedMovingAverage::new(TIME_CONSTANT, SAMPLE_RATE_HZ);

        let wave_cycle_status = WaveCycleStatus::Positive;
        let samples_since_zero_crossing = 0;
        let peak_threshold = Self::INITIAL_PEAK_THRESHOLD;

        TimecodeChannel {
            ewma,
            wave_cycle_status,
            samples_since_zero_crossing,
            peak_threshold,
        }
    }

    /// Returns true if the wave has crossed zero.
    pub fn has_crossed_zero(&self, sample: i32) -> bool {
        let adjusted_zero_crossing_threshold =
            self.ewma.last_output + Self::ZERO_CROSSING_THRESHOLD;
        match self.wave_cycle_status {
            WaveCycleStatus::Negative => sample > adjusted_zero_crossing_threshold,
            WaveCycleStatus::Positive => sample < adjusted_zero_crossing_threshold,
        }
    }

    /// Process a sample and detect zero crossing.
    pub fn process_sample(&mut self, sample: i32) -> bool {
        let crossed_zero = self.has_crossed_zero(sample);
        if crossed_zero {
            self.wave_cycle_status = match self.wave_cycle_status {
                WaveCycleStatus::Negative => WaveCycleStatus::Positive,
                WaveCycleStatus::Positive => WaveCycleStatus::Negative,
            };
            self.samples_since_zero_crossing = 0;
        } else {
            self.samples_since_zero_crossing += 1;
        }

        self.ewma.process(sample);

        crossed_zero
    }

    /// Reads a bit from the sample and adjust the threshold.
    pub fn bit_from_sample(&mut self, sample: i32) -> bool {
        let delta = self.ewma.difference_to(sample).abs() - self.peak_threshold;
        let bit = delta > 0;

        // TODO: The peak threshold is more or less determined by trial and error. This needs to be
        // improved by somebody with more DSP knowledge.
        self.peak_threshold += delta >> 6;

        bit
    }
}

#[derive(Debug)]
pub struct Timecode {
    bitstream: Bitstream,
    primary_channel: TimecodeChannel,
    secondary_channel: TimecodeChannel,
    direction: TimecodeDirection,
    pitch: PitchDetector,
}

impl Timecode {
    pub fn new(format: &TimecodeFormat) -> Self {
        let TimecodeFormat { size, seed, taps } = format;

        let bitstream = Bitstream::new(*size, *seed, *taps);
        let primary_channel = TimecodeChannel::new();
        let secondary_channel = TimecodeChannel::new();

        let pitch = PitchDetector::new(SAMPLE_RATE_HZ, TIMECODE_FREQUENCY_HZ);

        Self {
            bitstream,
            primary_channel,
            secondary_channel,
            direction: TimecodeDirection::Forwards,
            pitch,
        }
    }

    pub fn process_channels(
        &mut self,
        primary_sample: i16,
        secondary_sample: i16,
    ) -> Option<(bool, Option<u32>)> {
        let primary_sample = sample_to_i32(primary_sample);
        let secondary_sample = sample_to_i32(secondary_sample);
        let primary_crossed_zero = self.primary_channel.process_sample(primary_sample);
        let secondary_crossed_zero = self.secondary_channel.process_sample(secondary_sample);

        // detect playback direction
        if primary_crossed_zero {
            self.direction = if self.primary_channel.wave_cycle_status
                == self.secondary_channel.wave_cycle_status
            {
                TimecodeDirection::Forwards
            } else {
                TimecodeDirection::Backwards
            }
        } else if secondary_crossed_zero {
            self.direction = if self.primary_channel.wave_cycle_status
                != self.secondary_channel.wave_cycle_status
            {
                TimecodeDirection::Forwards
            } else {
                TimecodeDirection::Backwards
            }
        }

        // detect playback speed
        if primary_crossed_zero || secondary_crossed_zero {
            let pitch = self.pitch.update_after_zero_crossing(
                primary_sample,
                secondary_sample,
                primary_crossed_zero,
            );
            dbg!(pitch);
        } else {
            self.pitch.update(primary_sample, secondary_sample);
        }

        // Read a bit from the timecode and detect position within the timecode signal
        if secondary_crossed_zero
            && self.primary_channel.wave_cycle_status == WaveCycleStatus::Positive
        {
            let bit = self.primary_channel.bit_from_sample(primary_sample);
            if self.direction == TimecodeDirection::Forwards {
                self.bitstream.process_bit(bit as u32);
            } else {
                self.bitstream.process_bit_backward(bit as u32);
            }
            return Some((bit, self.bitstream.position()));
        }

        None
    }
}
