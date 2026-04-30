use std::ffi::{c_void, CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;

use tracing::{debug, info};

use super::ffi::MpvFfi;
use super::types::*;
use crate::error::MpvError;

type MpvHandle = *mut c_void;

/// Safe wrapper around a libmpv instance.
pub struct MpvPlayer {
    handle: MpvHandle,
}

// mpv API is thread-safe: commands/properties from any thread,
// mpv_wait_event from one dedicated thread.
unsafe impl Send for MpvPlayer {}
unsafe impl Sync for MpvPlayer {}

impl MpvPlayer {
    /// Create a new mpv instance attached to the given window handle.
    pub fn new(hwnd: i64) -> Result<Self, MpvError> {
        let ffi = MpvFfi::init()?;

        let handle = unsafe { (ffi.create)() };
        if handle.is_null() {
            return Err(MpvError::api(-1, "mpv_create returned null"));
        }

        info!(hwnd, "Creating mpv instance");

        // Set wid before initialize (required by mpv)
        Self::set_option_raw(ffi, handle, "wid", MPV_FORMAT_INT64, &hwnd)?;

        // Pre-init options (must be set before mpv_initialize)
        for (k, v) in [
            ("video-sync", "display-resync"),
            ("interpolation", "yes"),
            ("tscale", "oversample"),
        ] {
            Self::set_option_string_raw(ffi, handle, k, v);
        }

        let rc = unsafe { (ffi.initialize)(handle) };
        if rc < 0 {
            return Err(MpvError::api(rc, "mpv_initialize"));
        }

        let player = Self { handle };

        // Post-init configuration
        player.set::<&str>("hwdec", "auto-safe")?;
        player.set::<&str>("osd-level", "0")?;
        player.set::<&str>("keep-open", "yes")?;
        player.set::<&str>("idle", "yes")?;
        player.set::<&str>("input-default-bindings", "no")?;
        player.set::<&str>("input-vo-keyboard", "no")?;
        player.set::<&str>("osc", "no")?;
        player.set::<&str>("sub-auto", "fuzzy")?;

        info!("mpv instance created and configured");
        Ok(player)
    }

    /// Send a command to mpv (e.g. `["loadfile", "/path/to/file"]`).
    pub fn command(&self, args: &[&str]) -> Result<(), MpvError> {
        let ffi = MpvFfi::global()?;
        let c_args: Vec<CString> = args
            .iter()
            .map(|s| CString::new(*s).map_err(MpvError::from))
            .collect::<Result<_, _>>()?;
        let mut ptrs: Vec<*const c_char> = c_args.iter().map(|s| s.as_ptr()).collect();
        ptrs.push(ptr::null());

        let rc = unsafe { (ffi.command)(self.handle, ptrs.as_ptr()) };
        if rc < 0 {
            return Err(MpvError::api(rc, &format!("command {args:?}")));
        }
        debug!(args = ?args, "mpv command");
        Ok(())
    }

    /// Observe a property for changes (delivered via `wait_event`).
    pub fn observe_property(
        &self,
        name: &str,
        userdata: u64,
        format: c_int,
    ) -> Result<(), MpvError> {
        let ffi = MpvFfi::global()?;
        let c_name = CString::new(name)?;
        let rc = unsafe { (ffi.observe_property)(self.handle, userdata, c_name.as_ptr(), format) };
        if rc < 0 {
            return Err(MpvError::api(rc, &format!("observe_property({name})")));
        }
        Ok(())
    }

    /// Block until the next event (with timeout in seconds).
    pub fn wait_event(&self, timeout: f64) -> &MpvEvent {
        let ffi = MpvFfi::global().expect("FFI must be initialized before wait_event");
        unsafe { &*(ffi.wait_event)(self.handle, timeout) }
    }

    /// Get a typed property from mpv.
    pub fn get<T: MpvProperty>(&self, name: &str) -> Result<T, MpvError> {
        T::get_from(self.handle, name)
    }

    /// Set a typed property on mpv.
    pub fn set<T: MpvProperty>(&self, name: &str, value: T) -> Result<(), MpvError> {
        T::set_on(self.handle, name, value)
    }

    /// Get a property as string (convenience for reading any property).
    pub fn get_property_string(&self, name: &str) -> Result<String, MpvError> {
        let ffi = MpvFfi::global()?;
        let c_name = CString::new(name)?;
        let ptr = unsafe { (ffi.get_property_string)(self.handle, c_name.as_ptr()) };
        if ptr.is_null() {
            return Err(MpvError::api(-1, &format!("null string for '{name}'")));
        }
        let s = unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() };
        unsafe { (ffi.free)(ptr as *mut c_void) };
        Ok(s)
    }

    // --- Private helpers for pre-init options ---

    fn set_option_raw(
        ffi: &MpvFfi,
        handle: MpvHandle,
        name: &str,
        format: c_int,
        data: *const impl Sized,
    ) -> Result<(), MpvError> {
        let c_name = CString::new(name)?;
        let rc =
            unsafe { (ffi.set_option)(handle, c_name.as_ptr(), format, data as *const c_void) };
        if rc < 0 {
            return Err(MpvError::api(rc, &format!("set_option({name})")));
        }
        Ok(())
    }

    fn set_option_string_raw(ffi: &MpvFfi, handle: MpvHandle, name: &str, value: &str) {
        if let (Ok(k), Ok(v)) = (CString::new(name), CString::new(value)) {
            unsafe { (ffi.set_option_string)(handle, k.as_ptr(), v.as_ptr()) };
        }
    }
}

impl Drop for MpvPlayer {
    fn drop(&mut self) {
        if let Ok(ffi) = MpvFfi::global() {
            info!("Destroying mpv instance");
            unsafe { (ffi.terminate_destroy)(self.handle) };
        }
    }
}

// ---------------------------------------------------------------------------
// Typed property trait — compile-time type-safe mpv property access
// ---------------------------------------------------------------------------

/// Trait for types that can be read/written as mpv properties.
pub trait MpvProperty: Sized {
    fn get_from(handle: MpvHandle, name: &str) -> Result<Self, MpvError>;
    fn set_on(handle: MpvHandle, name: &str, value: Self) -> Result<(), MpvError>;
}

impl MpvProperty for f64 {
    fn get_from(handle: MpvHandle, name: &str) -> Result<Self, MpvError> {
        let ffi = MpvFfi::global()?;
        let c_name = CString::new(name)?;
        let mut val: f64 = 0.0;
        let rc = unsafe {
            (ffi.get_property)(
                handle,
                c_name.as_ptr(),
                MPV_FORMAT_DOUBLE,
                &mut val as *mut f64 as *mut c_void,
            )
        };
        if rc < 0 {
            return Err(MpvError::api(rc, name));
        }
        Ok(val)
    }

    fn set_on(handle: MpvHandle, name: &str, value: Self) -> Result<(), MpvError> {
        let ffi = MpvFfi::global()?;
        let c_name = CString::new(name)?;
        let rc = unsafe {
            (ffi.set_property)(
                handle,
                c_name.as_ptr(),
                MPV_FORMAT_DOUBLE,
                &value as *const f64 as *const c_void,
            )
        };
        if rc < 0 {
            return Err(MpvError::api(rc, name));
        }
        Ok(())
    }
}

impl MpvProperty for i64 {
    fn get_from(handle: MpvHandle, name: &str) -> Result<Self, MpvError> {
        let ffi = MpvFfi::global()?;
        let c_name = CString::new(name)?;
        let mut val: i64 = 0;
        let rc = unsafe {
            (ffi.get_property)(
                handle,
                c_name.as_ptr(),
                MPV_FORMAT_INT64,
                &mut val as *mut i64 as *mut c_void,
            )
        };
        if rc < 0 {
            return Err(MpvError::api(rc, name));
        }
        Ok(val)
    }

    fn set_on(handle: MpvHandle, name: &str, value: Self) -> Result<(), MpvError> {
        let ffi = MpvFfi::global()?;
        let c_name = CString::new(name)?;
        let rc = unsafe {
            (ffi.set_property)(
                handle,
                c_name.as_ptr(),
                MPV_FORMAT_INT64,
                &value as *const i64 as *const c_void,
            )
        };
        if rc < 0 {
            return Err(MpvError::api(rc, name));
        }
        Ok(())
    }
}

impl MpvProperty for bool {
    fn get_from(handle: MpvHandle, name: &str) -> Result<Self, MpvError> {
        let ffi = MpvFfi::global()?;
        let c_name = CString::new(name)?;
        let mut val: c_int = 0;
        let rc = unsafe {
            (ffi.get_property)(
                handle,
                c_name.as_ptr(),
                MPV_FORMAT_FLAG,
                &mut val as *mut c_int as *mut c_void,
            )
        };
        if rc < 0 {
            return Err(MpvError::api(rc, name));
        }
        Ok(val != 0)
    }

    fn set_on(handle: MpvHandle, name: &str, value: Self) -> Result<(), MpvError> {
        let ffi = MpvFfi::global()?;
        let c_name = CString::new(name)?;
        let flag: c_int = if value { 1 } else { 0 };
        let rc = unsafe {
            (ffi.set_property)(
                handle,
                c_name.as_ptr(),
                MPV_FORMAT_FLAG,
                &flag as *const c_int as *const c_void,
            )
        };
        if rc < 0 {
            return Err(MpvError::api(rc, name));
        }
        Ok(())
    }
}

/// String properties use `set_property_string` / `get_property_string`.
impl MpvProperty for &str {
    fn get_from(_handle: MpvHandle, _name: &str) -> Result<Self, MpvError> {
        // Can't return &str from FFI — use player.get_property_string() instead
        Err(MpvError::api(-1, "use get_property_string for string reads"))
    }

    fn set_on(handle: MpvHandle, name: &str, value: Self) -> Result<(), MpvError> {
        let ffi = MpvFfi::global()?;
        let c_name = CString::new(name)?;
        let c_value = CString::new(value)?;
        let rc = unsafe {
            (ffi.set_property_string)(handle, c_name.as_ptr(), c_value.as_ptr())
        };
        if rc < 0 {
            return Err(MpvError::api(rc, name));
        }
        Ok(())
    }
}
