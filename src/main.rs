use std::ffi::CString;
use std::thread::sleep;
use std::time::Duration;
use getopts;
use tokio;
use x11::xlib::{
    XDefaultScreen,
    XFlush,
    XOpenDisplay,
    XRootWindow,
    XStoreName
};

mod components;
use components::{exec, memory, time, battery};
mod config;
use config::{read_config, Item};

static VERSION: &str = "0.1.1";

impl Item {
    pub async fn process(&self, sep: &String) -> Option<String> {
        match &self.name[..] {
            "memory" => memory().await,
            "time" => time().await,
            "exec" => exec(&self.params).await,
            "battery" => battery(self.params.first().map(|x| x.to_owned())).await,
            "echo" => Some(self.params.join(" ")),
            "sep" => Some(sep.to_string()),
            name => {
                println!("{} module not implemented", name);
                None
            }
        }
    }
}

fn print_usage(program: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    println!("{}", opts.usage(&brief));
}

fn print_version() {
    println!("version: {}", VERSION);
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "display this help and exit");
    opts.optopt("c", "config", "config file path", "PATH");
    opts.optflag("v", "version", "output version information and exit");

    let mut conf = read_config(None);

    match opts.parse(&args[1..]) {
        Ok(m) => {
            if m.opt_present("h") {
                print_usage(&args[0].clone(), opts);
                return;
            }
            if m.opt_present("v") {
                print_version();
                return;
            }
            if m.opt_present("c") {
                let path = m.opt_str("c");
                conf = read_config(path);
            }
        },
        Err(_) => {},
    };

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
            let res = item.process(&conf.sep).await;
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
