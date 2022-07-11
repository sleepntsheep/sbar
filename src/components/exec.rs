use std::process::Command;

pub async fn exec(args: &Vec<String>) -> Option<String> {
    if args.len() >= 1 {
        let output = Command::new(&args[0])
            .args(&args[1..args.len()])
            .output()
            .expect("Failed to exec process");
        Some(String::from_utf8(output.stdout).unwrap().trim().to_owned())
    } else {
        None
    }
}
