use super::format::TimecodeFormat;
use super::lfsr::FibonacciLfsr;

#[derive(Debug, Clone, PartialEq)]
pub struct TimecodeAudioGenerator {
    lfsr: FibonacciLfsr,
    sample_rate_hz: f64,
    signal_frequency_hz: f64,
    previous_bit: bool,
    cycle_index: i32,
    index: i32,
}

impl TimecodeAudioGenerator {
    pub fn new(format: &TimecodeFormat, sample_rate_hz: f64) -> Self {
        let TimecodeFormat {
            size,
            seed,
            taps,
            signal_frequency_hz,
        } = format;

        let mut lfsr = FibonacciLfsr {
            size: *size,
            state: *seed,
            taps: *taps,
        };
        let signal_frequency_hz = *signal_frequency_hz;
        lfsr.revert();
        let previous_bit = (lfsr.state >> (lfsr.size - 2)) & 1 == 1;
        lfsr.advance();
        assert_eq!(lfsr.state, *seed);

        Self {
            lfsr,
            sample_rate_hz,
            signal_frequency_hz,
            cycle_index: 0,
            previous_bit,
            index: 0,
        }
    }

    const SCALE_FACTOR_ZERO: f64 = 0.75;

    fn scale_sample(sample: f64) -> i16 {
        let sample = sample * (i16::MAX as f64) * 0.5;
        sample.round().trunc() as i16
    }

    fn sample_from_cycle(cycle: f64, primary_bit: bool, secondary_bit: bool) -> (f64, f64) {
        let angle = 2.0 * std::f64::consts::PI * cycle;
        let (mut primary, mut secondary) = angle.sin_cos();

        if !primary_bit {
            primary *= Self::SCALE_FACTOR_ZERO;
        }

        if !secondary_bit {
            secondary *= Self::SCALE_FACTOR_ZERO;
        };

        (primary, secondary)
    }

    pub fn next_sample(&mut self) -> (i16, i16) {
        let index = f64::from(self.index);

        let cycle = (index * self.signal_frequency_hz) / self.sample_rate_hz;
        let cycle_index = cycle.trunc() as i32;
        let cycle_position = cycle - f64::from(cycle_index);

        if cycle_index == self.cycle_index && cycle_position >= 0.75 {
            self.cycle_index = cycle_index + 1;
            self.previous_bit = (self.lfsr.state >> (self.lfsr.size - 1)) & 1 == 1;
            self.lfsr.advance();
        }

        let secondary_bit = (self.lfsr.state >> (self.lfsr.size - 1)) == 1;
        let primary_bit = if cycle_position >= 0.75 {
            self.previous_bit
        } else {
            secondary_bit
        };

        let (mut primary_sample, mut secondary_sample) =
            Self::sample_from_cycle(cycle, primary_bit, secondary_bit);

        if cycle < 1.0 {
            primary_sample *= cycle;
            secondary_sample *= cycle;
        }

        let primary_sample = Self::scale_sample(primary_sample);
        let secondary_sample = Self::scale_sample(secondary_sample);

        self.index += 1;
        (primary_sample, secondary_sample)
    }

    pub fn state(&self) -> u32 {
        self.lfsr.state
    }
}

#[cfg(test)]
mod test {
    use super::TimecodeAudioGenerator;
    use crate::SERATO_CONTROL_CD_1_0_0;

    #[test]
    fn test_generator() {
        let mut generator = TimecodeAudioGenerator::new(&SERATO_CONTROL_CD_1_0_0, 44100.0);
        let initial_state = generator.state();
        loop {
            generator.next_sample();
            if generator.state() == initial_state {
                break;
            }
        }
    }
}
