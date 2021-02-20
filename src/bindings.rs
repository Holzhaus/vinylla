use super::{Timecode, SERATO_CONTROL_CD_1_0_0};
use std::mem::transmute;

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
/// Returns a pointer to the vinylla detection object.
pub unsafe extern "C" fn vinylla_init() -> *mut Timecode {
    transmute(Box::new(Timecode::new(&SERATO_CONTROL_CD_1_0_0)))
}

#[no_mangle]
/// Takes two samples and returns the detected position or -1.
///
/// # Safety
///
/// Passing invalid values or null pointers might result in a crash.
pub unsafe extern "C" fn vinylla_process(ptr: *mut Timecode, left: i16, right: i16) -> i64 {
    let timecode = &mut *ptr;
    timecode.process_channels(left, right).map_or(-1, i64::from)
}

#[no_mangle]
/// Query the last detected pitch value.
///
/// # Safety
///
/// Passing invalid values or null pointers might result in a crash.
pub unsafe extern "C" fn vinylla_pitch(ptr: *const Timecode) -> f64 {
    let timecode = &*ptr;
    timecode.pitch()
}
