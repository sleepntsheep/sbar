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

mod components;
use components::{battery, exec, memory, time};
mod config;
use config::{read_config, Bar, Item};

static VERSION: &str = "0.5.4";

static mut DPY: *mut Display = null_mut();

pub fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let tmp = a;
        a = b;
        b = tmp % b;
    }
    a
}

fn print_usage(program: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    println!("{}", opts.usage(&brief));
}

fn print_version() {
    println!("version: {}", VERSION);
}

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

async fn updateone(bar: &mut Bar, i: usize) {
    bar.list[i].str = bar.list[i].process(bar.sep.to_string()).await;
    draw(bar).await;
}

async fn updatebysig(this: Arc<Mutex<Bar>>, sig: i32) {
    let mtx = &mut this.lock().await;
    let bar = &mut (**mtx);
    for (idx, item) in bar.list.iter().enumerate() {
        if item.signal == sig {
            updateone(&mut bar.clone(), idx).await;
        }
    }
}

async fn draw(bar: &mut Bar) {
    let str = bar
        .list
        .iter()
        .filter(|x| x.str.is_some())
        .map(|x| x.clone().str.unwrap())
        .collect::<Vec<String>>()
        .join(if bar.autosep { bar.sep.as_str() } else { "" });
    let cstr = match CString::new(str) {
        Ok(r) => r,
        Err(err) => {
            eprintln!("error creating CString: {}", err);
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

    #[allow(unused_mut)]
    let mut bar = read_config(confpath);

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
