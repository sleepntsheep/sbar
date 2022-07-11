use futures::stream::StreamExt;
use getopts;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::ffi::CString;
use std::process::exit;
use std::ptr;
use std::sync::Arc;
use std::time::Duration;
use tokio;
use x11::xlib::{Display, XDefaultRootWindow, XFlush, XOpenDisplay, XStoreName};

mod components;
use components::{battery, exec, memory, time};
mod config;
use config::{read_config, Config, Item};

static VERSION: &str = "0.4.2";

static mut DPY: *mut Display = ptr::null_mut();

impl Item {
    pub async fn process(&self, sep: String) -> Option<String> {
        match self.name[..].trim() {
            "memory" => memory().await,
            "time" => time().await,
            "exec" => exec(&self.params).await,
            "battery" => battery(&self.params).await,
            "echo" => Some(self.params.join(" ")),
            "sep" => Some(sep),
            name => {
                println!("{} module not implemented", name);
                None
            }
        }
    }
}

#[derive(Clone)]
struct Sbar {
    pub conf: Config,
}

impl Sbar {
    async fn update(&self) {
        let mut str = String::new();
        for (idx, item) in self.conf.list.iter().enumerate() {
            if idx != 0 && self.conf.autosep {
                str += &self.conf.sep;
            }
            let res = item.process(self.conf.sep.to_string()).await;
            match res {
                Some(r) => {
                    str += &r;
                }
                None => {}
            }
        }
        unsafe {
            let cstr = CString::new(str).unwrap();
            XStoreName(DPY, XDefaultRootWindow(DPY), cstr.as_ptr());
            XFlush(DPY);
        }
    }

    async fn handle_signals(self: Arc<Self>, signals: Signals) {
        //async fn handle_signals(&self, signals: Signals) {
        let mut signals = signals.fuse();
        while let Some(signal) = signals.next().await {
            match signal {
                SIGHUP => {
                    // Reload configuration
                    // Reopen the log file
                }
                SIGTERM | SIGINT | SIGQUIT => exit(0),
                _ => {
                    self.update().await;
                }
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
    let mut confpath: Option<String> = None;

    // getopt
    let args: Vec<String> = std::env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "display this help and exit");
    opts.optflag("v", "version", "output version information and exit");
    opts.optopt("c", "config", "config file path", "PATH");
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
                confpath = m.opt_str("c");
            }
        }
        Err(_) => {}
    };

    let bar = Sbar {
        conf: read_config(confpath),
    };

    unsafe {
        let dpy_n = 0_i8;
        DPY = XOpenDisplay(&dpy_n);
        if DPY.is_null() {
            panic!("Failed opening display");
        }
        DPY
    };

    // Signal
    let mut signals = vec![SIGHUP, SIGTERM, SIGINT, SIGQUIT];
    for item in bar.conf.list.iter() {
        if item.signal != 0 {
            signals.push(item.signal);
        }
    }
    let signalsinfo = Signals::new(signals).unwrap();

    let _handle = signalsinfo.handle();
    let _signals_task = tokio::spawn(Arc::from(bar.clone()).handle_signals(signalsinfo));

    // main loop
    loop {
        bar.update().await;
        tokio::time::sleep(Duration::from_millis(bar.conf.delay)).await;
    }

    //_handle.
    //signals_task.await;
}
