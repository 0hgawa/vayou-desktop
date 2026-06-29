use std::ffi::c_void;
use std::os::raw::{c_char, c_double, c_int};
use std::sync::OnceLock;

use libloading::Library;
use tracing::info;

use super::types::MpvEvent;
use crate::error::MpvError;

type MpvHandle = *mut c_void;

/// All mpv function pointers, resolved once at startup.
/// The `Library` is kept alive to ensure pointers remain valid.
pub struct MpvFfi {
    _lib: Library,
    pub create: unsafe extern "C" fn() -> MpvHandle,
    pub initialize: unsafe extern "C" fn(MpvHandle) -> c_int,
    pub terminate_destroy: unsafe extern "C" fn(MpvHandle),
    pub set_option: unsafe extern "C" fn(MpvHandle, *const c_char, c_int, *const c_void) -> c_int,
    pub set_option_string:
        unsafe extern "C" fn(MpvHandle, *const c_char, *const c_char) -> c_int,
    pub command: unsafe extern "C" fn(MpvHandle, *const *const c_char) -> c_int,
    pub set_property:
        unsafe extern "C" fn(MpvHandle, *const c_char, c_int, *const c_void) -> c_int,
    pub set_property_string:
        unsafe extern "C" fn(MpvHandle, *const c_char, *const c_char) -> c_int,
    pub get_property:
        unsafe extern "C" fn(MpvHandle, *const c_char, c_int, *mut c_void) -> c_int,
    pub get_property_string: unsafe extern "C" fn(MpvHandle, *const c_char) -> *mut c_char,
    pub observe_property:
        unsafe extern "C" fn(MpvHandle, u64, *const c_char, c_int) -> c_int,
    pub wait_event: unsafe extern "C" fn(MpvHandle, c_double) -> *mut MpvEvent,
    pub free: unsafe extern "C" fn(*mut c_void),
}

// Function pointers are just addresses — safe to share across threads.
unsafe impl Send for MpvFfi {}
unsafe impl Sync for MpvFfi {}

static FFI: OnceLock<MpvFfi> = OnceLock::new();

impl MpvFfi {
    /// Get the global FFI instance, or error if not yet initialized.
    pub fn global() -> Result<&'static Self, MpvError> {
        FFI.get().ok_or(MpvError::NotInitialized)
    }

    /// Load libmpv and resolve all symbols. Idempotent — only loads once.
    pub fn init() -> Result<&'static Self, MpvError> {
        if let Some(ffi) = FFI.get() {
            return Ok(ffi);
        }
        let ffi = Self::load()?;
        let _ = FFI.set(ffi);
        FFI.get().ok_or(MpvError::NotInitialized)
    }

    fn load() -> Result<Self, MpvError> {
        info!("Loading libmpv");

        // We bundle libmpv-2.dll next to the binary; `mpv-2.dll` is kept as a
        // fallback name. The first that loads wins.
        const LIB_CANDIDATES: &[&str] = &["libmpv-2.dll", "mpv-2.dll"];

        let lib = unsafe {
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.to_path_buf()));
            let mut loaded = None;
            // 1. exe dir + binaries/ subfolder (where Windows bundle ships the .dll)
            if let Some(dir) = exe_dir.as_ref() {
                for name in LIB_CANDIDATES {
                    if let Ok(l) = Library::new(dir.join(name)).or_else(|_| Library::new(dir.join("binaries").join(name))) {
                        loaded = Some(l);
                        break;
                    }
                }
            }
            // 2. system loader (PATH / system dirs) as a fallback
            if loaded.is_none() {
                for name in LIB_CANDIDATES {
                    if let Ok(l) = Library::new(name) {
                        loaded = Some(l);
                        break;
                    }
                }
            }
            match loaded {
                Some(lib) => lib,
                None => return Err(MpvError::LibraryLoad(
                    format!("none of {LIB_CANDIDATES:?} could be loaded"),
                )),
            }
        };

        unsafe {
            let ffi = MpvFfi {
                create: *lib.get(b"mpv_create").map_err(|e| MpvError::symbol("mpv_create", e))?,
                initialize: *lib.get(b"mpv_initialize").map_err(|e| MpvError::symbol("mpv_initialize", e))?,
                terminate_destroy: *lib.get(b"mpv_terminate_destroy").map_err(|e| MpvError::symbol("mpv_terminate_destroy", e))?,
                set_option: *lib.get(b"mpv_set_option").map_err(|e| MpvError::symbol("mpv_set_option", e))?,
                set_option_string: *lib.get(b"mpv_set_option_string").map_err(|e| MpvError::symbol("mpv_set_option_string", e))?,
                command: *lib.get(b"mpv_command").map_err(|e| MpvError::symbol("mpv_command", e))?,
                set_property: *lib.get(b"mpv_set_property").map_err(|e| MpvError::symbol("mpv_set_property", e))?,
                set_property_string: *lib.get(b"mpv_set_property_string").map_err(|e| MpvError::symbol("mpv_set_property_string", e))?,
                get_property: *lib.get(b"mpv_get_property").map_err(|e| MpvError::symbol("mpv_get_property", e))?,
                get_property_string: *lib.get(b"mpv_get_property_string").map_err(|e| MpvError::symbol("mpv_get_property_string", e))?,
                observe_property: *lib.get(b"mpv_observe_property").map_err(|e| MpvError::symbol("mpv_observe_property", e))?,
                wait_event: *lib.get(b"mpv_wait_event").map_err(|e| MpvError::symbol("mpv_wait_event", e))?,
                free: *lib.get(b"mpv_free").map_err(|e| MpvError::symbol("mpv_free", e))?,
                _lib: lib,
            };

            info!("libmpv loaded — all symbols resolved");
            Ok(ffi)
        }
    }
}
