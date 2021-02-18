use crate::{bitstream::Bitstream, format::TimecodeFormat, util::ExponentialWeightedMovingAverage};

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
}

impl Timecode {
    pub fn new(format: &TimecodeFormat) -> Self {
        let TimecodeFormat {
            size,
            seed,
            taps,
        } = format;

        let bitstream = Bitstream::new(*size, *seed, *taps);
        let primary_channel = TimecodeChannel::new();
        let secondary_channel = TimecodeChannel::new();

        Self {
            bitstream,
            primary_channel,
            secondary_channel,
            direction: TimecodeDirection::Forwards,
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

        // Detect the playback direction of the timecode.
        //
        //                         Assuming the primary channel crossed zero:
        //  ──╮   ╭───╮   ╭(4)╮    If both the primary wave and the secondary
        //    │  (2)  │   │   │    wave are negative (1) or both are postive
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
