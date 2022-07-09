use std::ffi::CString;
use std::thread::sleep;
use std::time::Duration;
use tokio;
use x11::xlib::{
    XDefaultScreen,
    XFlush,
    XOpenDisplay,
    XRootWindow,
    XStoreName
};

mod components;
use components::{exec, memory, time};
mod config;
use config::{read_config, Item};

impl Item {
    pub async fn process(&self) -> Option<String> {
        match &self.name[..] {
            "memory" => memory().await,
            "time" => time().await,
            "exec" => exec(&self.params).await,
            name => {
                println!("{} module not implemented", name);
                None
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let conf = read_config();

    let dpy_n = 0_i8;
    let dpy = unsafe { XOpenDisplay(&dpy_n) };
    if dpy.is_null() {
        panic!("Failed opening display");
    }
    let scr = unsafe { XDefaultScreen(dpy) };
    let root = unsafe { XRootWindow(dpy, scr) };

    loop {
        let mut str = String::new();
        for (idx, item) in conf.list.iter().enumerate() {
            if idx != 0 {
                str += &conf.sep;
            }
            let res = item.process().await;
            match res {
                Some(r) => {
                    str += &r;
                }
                None => {}
            }
        }
        let cstr = CString::new(str).unwrap();
        unsafe {
            if (XStoreName(dpy, root, cstr.as_ptr())) < 0 {
                panic!("XStoreName failed");
            }
            XFlush(dpy);
        }
        sleep(Duration::from_millis(conf.delay));
    }
}
