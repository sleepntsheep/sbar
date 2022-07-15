use std::process::Command;
use log::info;

pub async fn exec(args: &Vec<String>) -> Option<String> {
    if args.len() >= 1 {
        let output = match Command::new(&args[0])
            .args(&args[1..args.len()])
            .output() {
                Ok(r) => r,
                Err(err) => {
                    info!("exec: failed to exec {} ({})", args[0], err);
                    return None;
                }
            };
        Some(String::from_utf8(output.stdout).unwrap().trim().to_owned())
    } else {
        None
    }
}
