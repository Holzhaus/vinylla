// FIXME: Enable missing_docs
//#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(broken_intra_doc_links)]
#![cfg_attr(test, deny(warnings))]

pub mod bindings;
mod bits;
mod bitstream;
mod format;
mod lfsr;
mod pitch;
mod timecode;
mod util;

pub use format::SERATO_CONTROL_CD_1_0_0;
pub use timecode::Timecode;
