use futures::stream::StreamExt;
use getopts;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::ffi::CString;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use tokio;
use x11::xlib::{XDefaultRootWindow, XFlush, XOpenDisplay, XStoreName};

mod components;
use components::{battery, exec, memory, time};
mod config;
use config::{read_config, Config, Item};

static VERSION: &str = "0.3.2";

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
            setbar(str);
        }
    }

    async fn handle_signals(self: Arc<Self>, signals: Signals) {
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

unsafe fn setbar(str: String) {
    // X11
    let dpy_n = 0_i8;
    let dpy = XOpenDisplay(&dpy_n);
    if dpy.is_null() {
        panic!("Failed opening display");
    }
    let cstr = CString::new(str).unwrap();
    XStoreName(dpy, XDefaultRootWindow(dpy), cstr.as_ptr());
    XFlush(dpy);
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

    // Signal
    let mut signals = vec![SIGHUP, SIGTERM, SIGINT, SIGQUIT];
    for item in bar.conf.list.iter() {
        if item.signal != 0 {
            signals.push(item.signal);
        }
    }
    let signalsinfo = Signals::new(signals).unwrap();

    let _handle = signalsinfo.handle();
    {
        let bar2 = Arc::from(bar.clone());
        let _signals_task = tokio::spawn(bar2.handle_signals(signalsinfo));
    }

    // main loop
    loop {
        bar.update().await;
        tokio::time::sleep(Duration::from_millis(bar.conf.delay)).await;
    }

    //_handle.
    //signals_task.await;
}
