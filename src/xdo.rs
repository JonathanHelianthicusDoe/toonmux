//! This module is where all of the gross `unsafe` stuff lives.

use gdk::{self, keys::Key};
use glib::GString;
use libxdo_sys;
use std::{num::NonZeroI32, os::raw::c_char};
use x11::xlib::Window;

#[derive(Debug)]
pub struct Xdo {
    handle: *mut libxdo_sys::xdo_t,
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

    #[inline]
    pub fn send_key_down(
        &self,
        window: Window,
        key: &Key,
    ) -> Result<(), NonZeroI32> {
        if window == 0 {
            return Ok(());
        }

        let keyval_name = key.name().expect("invalid `Key`");
        let res = unsafe {
            libxdo_sys::xdo_send_keysequence_window_down(
                self.handle,
                window,
                gstring_as_ptr(&keyval_name),
                0,
            )
        };

        if let Some(code) = NonZeroI32::new(res) {
            Err(code)
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn send_key_up(
        &self,
        window: Window,
        key: &Key,
    ) -> Result<(), NonZeroI32> {
        if window == 0 {
            return Ok(());
        }

        let keyval_name = key.name().expect("invalid `Key`");
        let res = unsafe {
            libxdo_sys::xdo_send_keysequence_window_up(
                self.handle,
                window,
                gstring_as_ptr(&keyval_name),
                0,
            )
        };

        if let Some(code) = NonZeroI32::new(res) {
            Err(code)
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn send_key(
        &self,
        window: Window,
        key: &Key,
    ) -> Result<(), NonZeroI32> {
        if window == 0 {
            return Ok(());
        }

        let keyval_name = key.name().expect("invalid `Key`");
        let res = unsafe {
            libxdo_sys::xdo_send_keysequence_window(
                self.handle,
                window,
                gstring_as_ptr(&keyval_name),
                0,
            )
        };

        if let Some(code) = NonZeroI32::new(res) {
            Err(code)
        } else {
            Ok(())
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

/// Return value has the same lifetime as `gstring`.
#[inline(always)]
fn gstring_as_ptr(gstring: &GString) -> *const c_char {
    gstring.as_str().as_ptr() as *const c_char
}
