use tokio::fs;
use log::info;


pub async fn temp(args: &Vec<String>) -> Option<String> {
    let zone: i32 = match args.first() {
        Some(p) => match p.trim().parse::<i32>() {
            Ok(p) => p,
            Err(err) => {
                info!("battery: error parsing args, using bat0: {}", err);
                0
            }
        },
        None => 0,
    };

    let temp = match fs::read_to_string(format!("/sys/class/thermal/thermal_zone{}/temp", zone)).await
    {
        Ok(st) => {
            match st.trim().parse::<i32>() {
                Ok(r) => r / 1000,
                Err(err) => {
                    info!("temp: error reading parsing temp {}", err);
                    return None;
                }
            }
        }
        Err(err) => {
            info!("temp: error reading temp {}", err);
            return None;
        }
    };

    Some(format!("{}", temp))
}
