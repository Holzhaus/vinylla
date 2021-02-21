// vinylla - (c) 2021 Jan Holthuis <holthuis.jan@gmail.com> et al.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// FIXME: Enable missing_docs
//#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(broken_intra_doc_links)]
#![cfg_attr(test, deny(warnings))]

mod bits;
mod bitstream;
mod format;
mod generator;
mod lfsr;
mod pitch;
mod timecode;
mod util;
mod visualizer;

pub use format::SERATO_CONTROL_CD_1_0_0;
pub use generator::TimecodeAudioGenerator;
pub use timecode::Timecode;
pub use visualizer::Visualizer;
