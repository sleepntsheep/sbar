use sysinfo::{System, SystemExt};

pub async fn memory() -> Option<String> {
    let mut sys = System::new();
    sys.refresh_memory();
    Some(format!("{:.1} Gi", sys.used_memory() as f64 * 1e-6)) // convert KB to GB
}
