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
    let mut args = env::args().skip(1);
    let path = args.next().expect("No file given");
    let reverse = args.next().map_or(false, |x| x == "-r" || x == "--reverse");
    println!("Reverse: {}", reverse);

    println!("{}", path);
    let mut reader = WavReader::open(&path).unwrap();
    let mut timecode = Timecode::from(&SERATO_CONTROL_CD_1_0_0);

    let mut i = 0;
    if reverse {
        let mut position = reader.len() / 2;
        loop {
            if position < 2 {
                return;
            }
            reader.seek(position - 2).unwrap();
            let mut samples = reader.samples::<i16>().map(|x| x.unwrap());
            let left = samples.next().expect("failed to read left sample");
            let right = samples.next().expect("failed to read right sample");
            if let Some((bit, position)) = timecode.process_channels(left, right) {
                println!("{:10}: Bit {} => Position {:?}", i, bit as u8, position);
                i += 1;
            }
            position -= 2;
        }
    } else {
        let mut samples = reader.samples::<i16>().map(|x| x.unwrap());
        loop {
            let left = samples.next().expect("failed to read left sample");
            let right = samples.next().expect("failed to read right sample");
            if let Some((bit, position)) = timecode.process_channels(left, right) {
                println!("{:10}: Bit {} => Position {:?}", i, bit as u8, position);
                i += 1;
            }
        }
    }
}
