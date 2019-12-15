//! This module is where all of the gross `unsafe` stuff lives.

use gdk::{self, enums::key::Key};
use glib::GString;
use libxdo_sys;
use std::{ffi::CStr, num::NonZeroI32, os::raw::c_char};
use x11::xlib::Window;

#[derive(Debug)]
pub struct Xdo {
    handle: *mut libxdo_sys::xdo_t,
}

#[derive(Debug)]
enum XdoKeyName {
    Static(&'static CStr),
    GString(GString),
}

impl Xdo {
    pub fn new() -> Option<Self> {
        let handle = unsafe { libxdo_sys::xdo_new(::std::ptr::null()) };
        if handle.is_null() {
            None
        } else {
            Some(Self { handle })
        }
    }

    pub fn select_window_with_click(&self) -> Option<Window> {
        let mut window = Default::default();
        let res = unsafe {
            libxdo_sys::xdo_select_window_with_click(self.handle, &mut window)
        };

        if res == 0 {
            Some(window)
        } else {
            None
        }
    }

    pub fn send_key_down(
        &self,
        window: Window,
        key: Key,
    ) -> Result<(), NonZeroI32> {
        if window == 0 {
            return Ok(());
        }

        let key_name = Self::key_name(key);
        let res = unsafe {
            libxdo_sys::xdo_send_keysequence_window_down(
                self.handle,
                window,
                key_name.as_ptr(),
                0,
            )
        };

        if let Some(code) = NonZeroI32::new(res) {
            Err(code)
        } else {
            Ok(())
        }
    }

    pub fn send_key_up(
        &self,
        window: Window,
        key: Key,
    ) -> Result<(), NonZeroI32> {
        if window == 0 {
            return Ok(());
        }

        let key_name = Self::key_name(key);
        let res = unsafe {
            libxdo_sys::xdo_send_keysequence_window_up(
                self.handle,
                window,
                key_name.as_ptr(),
                0,
            )
        };

        if let Some(code) = NonZeroI32::new(res) {
            Err(code)
        } else {
            Ok(())
        }
    }

    pub fn send_key(
        &self,
        window: Window,
        key: Key,
    ) -> Result<(), NonZeroI32> {
        if window == 0 {
            return Ok(());
        }

        let key_name = Self::key_name(key);
        let res = unsafe {
            libxdo_sys::xdo_send_keysequence_window(
                self.handle,
                window,
                key_name.as_ptr(),
                0,
            )
        };

        if let Some(code) = NonZeroI32::new(res) {
            Err(code)
        } else {
            Ok(())
        }
    }

    fn key_name(key: Key) -> XdoKeyName {
        use XdoKeyName::*;

        match key {
            gdk::enums::key::uparrow => {
                Static(unsafe { CStr::from_bytes_with_nul_unchecked(b"Up\0") })
            }
            gdk::enums::key::downarrow => Static(unsafe {
                CStr::from_bytes_with_nul_unchecked(b"Down\0")
            }),
            gdk::enums::key::leftarrow => Static(unsafe {
                CStr::from_bytes_with_nul_unchecked(b"Left\0")
            }),
            gdk::enums::key::rightarrow => Static(unsafe {
                CStr::from_bytes_with_nul_unchecked(b"Right\0")
            }),
            _ => GString(gdk::keyval_name(key).expect("invalid `Key`")),
        }
    }
}

impl Drop for Xdo {
    fn drop(&mut self) {
        unsafe {
            libxdo_sys::xdo_free(self.handle);
        }
    }
}

impl XdoKeyName {
    /// Return value has same lifetime as `&self`.
    fn as_ptr(&self) -> *const c_char {
        match self {
            Self::Static(cstr) => cstr.as_ptr(),
            Self::GString(gstring) => gstring_as_ptr(gstring),
        }
    }
}

/// Return value has same lifetime as `gstring`.
fn gstring_as_ptr(gstring: &GString) -> *const c_char {
    match gstring {
        GString::ForeignOwned(maybe_cstring) => maybe_cstring
            .as_ref()
            .expect("ForeignOwned shouldn't be empty")
            .as_c_str()
            .as_ptr(),
        &GString::Borrowed(ptr, _) => ptr,
        &GString::Owned(mut_ptr, _) => mut_ptr as *const c_char,
    }
}
