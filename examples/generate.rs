//! Reads a Serato Control CD 1.0.0 WAV file and prints the decoded positions.
//!
//! The WAV file can be downloaded from:
//! https://serato.com/controlcd/downloads/zip
//!
//! You can run this using:
//!
//! ```bash
//! $ cargo run --example serato -- /path/to/Serato\ Control\ CD.wav
//! ```
//!
//! Note that this will panic when the end of the file is reached.

use hound::{SampleFormat, WavSpec, WavWriter};
use std::env;
use vinylla::{TimecodeAudioGenerator, SERATO_CONTROL_CD_1_0_0};

const SAMPLE_RATE_HZ: f64 = 44100.0;

fn main() {
    let mut args = env::args().skip(1);
    let path = args.next().expect("No file given");
    println!("{}", path);

    let spec = WavSpec {
        channels: 2,
        sample_rate: SAMPLE_RATE_HZ as u32,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut writer = WavWriter::create(&path, spec).unwrap();
    let mut generator = TimecodeAudioGenerator::new(&SERATO_CONTROL_CD_1_0_0, SAMPLE_RATE_HZ);
    let initial_state = generator.state();
    let mut state_changed = false;

    loop {
        let (left, right) = generator.next_sample();
        writer.write_sample(left).unwrap();
        writer.write_sample(right).unwrap();
        if !state_changed {
            state_changed = generator.state() != initial_state;
        } else if generator.state() == initial_state {
            break;
        }
    }
    writer.finalize().unwrap();
}
