mod time;
mod memory;
pub static ALL: [fn() -> Option<String>; 2]= [memory::memory, time::time];
pub static SEP: &str = " | ";