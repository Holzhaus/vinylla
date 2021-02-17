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

use hound::WavReader;
use std::env;
use vinylla::{Timecode, SERATO_CONTROL_CD_1_0_0};

fn main() {
    let path = env::args().skip(1).next().expect("No file given");
    println!("{}", path);
    let mut reader = WavReader::open(&path).unwrap();
    let mut samples = reader.samples::<i16>().map(|x| x.unwrap());

    let mut timecode = Timecode::from(&SERATO_CONTROL_CD_1_0_0);

    let mut i = 0;
    loop {
        let left = samples.next().expect("failed to read left sample");
        let right = samples.next().expect("failed to read right sample");
        if let Some((bit, position)) = timecode.process_channels(left, right) {
            println!("{:10}: Bit {} => Position {:?}", i, bit as u8, position);
            i += 1;
        }
    }
}
