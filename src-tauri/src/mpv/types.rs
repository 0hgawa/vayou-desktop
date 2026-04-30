use std::ffi::c_void;
use std::os::raw::{c_char, c_int};

/// mpv property format identifiers.
pub const MPV_FORMAT_STRING: c_int = 1;
pub const MPV_FORMAT_FLAG: c_int = 3;
pub const MPV_FORMAT_INT64: c_int = 4;
pub const MPV_FORMAT_DOUBLE: c_int = 5;

/// mpv event identifiers.
pub const MPV_EVENT_SHUTDOWN: c_int = 1;
pub const MPV_EVENT_FILE_LOADED: c_int = 8;
pub const MPV_EVENT_END_FILE: c_int = 7;
pub const MPV_EVENT_PROPERTY_CHANGE: c_int = 16;

/// Raw mpv event as returned by `mpv_wait_event`.
#[repr(C)]
pub struct MpvEvent {
    pub event_id: c_int,
    pub error: c_int,
    pub reply_userdata: u64,
    pub data: *mut c_void,
}

/// Property change data within an mpv event.
#[repr(C)]
pub struct MpvEventProperty {
    pub name: *const c_char,
    pub format: c_int,
    pub data: *mut c_void,
}
