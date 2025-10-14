// Copyright (c) 2025 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

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
    let reverse = args.next().is_some_and(|x| x == "-r" || x == "--reverse");
    println!("Reverse: {}", reverse);

    println!("{}", path);
    let mut reader = WavReader::open(&path).unwrap();
    let mut timecode = Timecode::new(&SERATO_CONTROL_CD_1_0_0, 44100.0);

    let mut i = 0;
    let mut position = reader.len() / 2;
    loop {
        if reader.len() < 2 || reverse && position < 2 {
            return;
        }

        if reverse {
            reader.seek(position - 2).unwrap();
            position -= 2;
        } else {
            position += 2;
        }
        let mut samples = reader.samples::<i16>().map(|x| x.unwrap());
        let left = match samples.next() {
            None => return,
            Some(s) => s,
        };
        let right = match samples.next() {
            None => return,
            Some(s) => s,
        };
        if let Some((bit, position)) = timecode.process_channels(left, right) {
            println!("{:10}: Bit {} => Position {:?}", i, bit as u8, position);
            i += 1;
        }
    }
}
