use futures::stream::StreamExt;
use getopts;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::ffi::CString;
use std::process::exit;
use std::ptr::null_mut;
use std::sync::Arc;
use std::time::Duration;
use tokio;
use tokio::sync::Mutex;
use x11::xlib::{Display, XDefaultRootWindow, XFlush, XOpenDisplay, XStoreName};
use log::{info, trace};

mod components;
use components::{battery, exec, memory, time, temp};
mod config;
use config::{read_config, Bar, Item};

static VERSION: &str = "0.7.6";

static mut DPY: *mut Display = null_mut();

fn print_usage(program: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    println!("{}", opts.usage(&brief));
}

fn print_version() {
    println!("version: {}", VERSION);
}

impl Item {
    pub async fn process(&self, bar: &Bar) -> Option<String> {
        let color_fg = match &self.fg {
            Some(c) => format!("^c{}^", c),
            None => "".to_string(),
        };
        let color_bg = match &self.bg {
            Some(c) => format!("^b{}^", c),
            None => "".to_string(),
        };
        let color_rst = "^d^";

        let prefix: String = {
            if self.prefix != "" { &self.prefix } else { &bar.prefix }
        }.clone();
        let suffix: String = {
            if self.suffix != "" { &self.suffix } else { &bar.suffix }
        }.clone();

        let txt = &match self.name[..].trim() {
            "memory" => memory::memory().await,
            "temp" => temp::temp(&self.params).await,
            "time" => time::time(&self.params).await,
            "exec" => exec::exec(&self.params).await,
            "battery" => battery::battery(&self.params).await,
            "battery_icon" => battery::battery_icon(&self.params).await,
            "echo" => Some(self.params.join(" ")),
            name => {
                info!("{} module not implemented", name);
                return None
            }
        }?;
        let mut txt = prefix + txt + &suffix;
        if bar.status2d_color {
            txt = color_fg + &color_bg + &txt + color_rst;
        }
        Some(txt)
    }
}

async fn updateone(bar: &mut Bar, i: usize) {
    bar.list[i].str = bar.list[i].process(bar).await;
}

async fn updatebysig(this: Arc<Mutex<Bar>>, sig: i32) {
    let mtx = &mut this.lock().await;
    let bar = &mut (**mtx);
    for (idx, item) in bar.clone().list.iter().enumerate() {
        if item.signal == sig {
            updateone(bar, idx).await;
            draw(bar).await;
        }
    }
}

async fn draw(bar: &mut Bar) {
    let str = bar
        .list
        .iter()
        .filter(|x| x.str.is_some())
        .map(|x| x.str.clone().unwrap())
        .collect::<Vec<String>>()
        .join(&bar.sep);
    let cstr = match CString::new(str) {
        Ok(r) => r,
        Err(err) => {
            info!("draw: failed create CString ({})", err);
            return;
        }
    };
    unsafe {
        XStoreName(DPY, XDefaultRootWindow(DPY), cstr.as_ptr());
        XFlush(DPY);
    }
}

async fn update(this: Arc<Mutex<Bar>>) {
    let mtx = &mut this.lock().await;
    let bar = &mut (**mtx);
    let mut drawing = false;
    if bar.counter == 0 {
        drawing = true;
        for (idx, _item) in bar.clone().list.iter_mut().enumerate() {
            updateone(bar, idx).await;
        }
    } else {
        for (idx, item) in bar.clone().list.iter().enumerate() {
            if item.interval != 0 && bar.counter % item.interval == 0 {
                drawing = true;
                updateone(bar, idx).await;
            }
        }
    }
    if drawing { draw(bar).await }
    bar.counter += 1;
}

async fn handle_signals(this: Arc<Mutex<Bar>>, signals: Signals) {
    let mut signals = signals.fuse();
    while let Some(signal) = signals.next().await {
        match signal {
            SIGHUP => {
                // Reload configuration
                // Reopen the log file
            }
            SIGTERM | SIGINT | SIGQUIT => exit(0),
            _ => {
                trace!("handle_signals: signal {} received", signal);
                updatebysig(this.clone(), signal).await;
            }
        }
    }
}

async fn run(this: Arc<Mutex<Bar>>) {
    loop {
        update(this.clone()).await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let mtx = &mut this.lock().await;
        let bar = &mut (**mtx);
        bar.counter += 1;
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut confpath: Option<String> = None;

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

    // init XDisplay
    unsafe {
        DPY = {
            let dpy_n = 0_i8;
            let dpy = XOpenDisplay(&dpy_n);
            if dpy.is_null() {
                panic!("Failed opening display");
            }
            dpy
        };
    }

    let bar = read_config(confpath);

    let mut signals = vec![SIGHUP, SIGTERM, SIGINT, SIGQUIT];
    for item in bar.list.iter() {
        if item.signal != 0 {
            signals.push(item.signal);
        }
    }
    let signalsinfo = Signals::new(signals).unwrap();
    let _handle = signalsinfo.handle();

    let barm = Arc::from(Mutex::new(bar));

    let signals_task = tokio::spawn(handle_signals(barm.clone(), signalsinfo));

    run(barm).await;

    signals_task.await.ok();
}
