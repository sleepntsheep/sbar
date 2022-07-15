use tokio::fs;
use log::info;

pub async fn battery(args: &Vec<String>) -> Option<String> {
    let bat: i32 = match args.first() {
        Some(p) => match p.trim().parse::<i32>() {
            Ok(p) => p,
            Err(err) => {
                info!("battery: error parsing args, using bat0: {}", err);
                0
            }
        },
        None => 0,
    };

    let perc =
        match fs::read_to_string(format!("/sys/class/power_supply/BAT{}/capacity", bat,)).await {
            Ok(p) => p,
            Err(err) => {
                info!("battery: error reading capacity: {}", err);
                return None;
            }
        };

    let perc = match perc.trim().parse::<i32>() {
        Ok(perc) => perc,
        Err(err) => {
            info!("battery: error parsing capacity, {}", err);
            return None;
        }
    };

    Some(format!("{}", perc))
}


pub async fn battery_icon(args: &Vec<String>) -> Option<String> {
    let bat: i32 = match args.first() {
        Some(p) => match p.trim().parse::<i32>() {
            Ok(p) => p,
            Err(err) => {
                info!("battery: error parsing args, using bat0: {}", err);
                0
            }
        },
        None => 0,
    };

    let stat = match fs::read_to_string(format!("/sys/class/power_supply/BAT{}/status", bat)).await
    {
        Ok(st) => st,
        Err(err) => {
            info!("battery: error reading status {}", err);
            return None;
        }
    };

    let icon = if stat == "Not charging" { "" } else { "" };

    Some(format!("{}", icon))
}
