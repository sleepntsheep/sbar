extern crate x11;
extern crate libc;

use std::thread::sleep;
use std::time::Duration;
use std::ffi::CString;
use x11::xlib;

mod components;
use components::{ALL, SEP};

fn main() {
    let delay_ms = 1000;
    let dpy_n = 0_i8;
    let dpy = unsafe { xlib::XOpenDisplay(&dpy_n) };
    if dpy.is_null() {
        panic!("Failed opening display");
    }
    let scr = unsafe { xlib::XDefaultScreen(dpy) };
    let root = unsafe { xlib::XRootWindow(dpy, scr) };

//    libc::signal()

    loop {
        unsafe {
            let mut str = String::new();
            for module in ALL.iter() {
                let res = module();
                match res {
                    Some(u) => {
                        str.push_str(&u);
                    },
                    None => {},
                }
                if module != ALL.last().unwrap() {
                    str.push_str(SEP);
                }
            }
            let cstr = CString::new(str).unwrap();
            if (xlib::XStoreName(dpy, root, cstr.as_ptr())) < 0 {
                panic!("XStoreName failed");
            }
            xlib::XFlush(dpy);
        }
        sleep(Duration::from_millis(delay_ms));
    }
}
