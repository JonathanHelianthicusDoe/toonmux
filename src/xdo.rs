//! This module is where all of the gross `unsafe` stuff lives.

use libxdo_sys;
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
}

impl Drop for Xdo {
    fn drop(&mut self) {
        unsafe {
            libxdo_sys::xdo_free(self.handle);
        }
    }
}
