use tokio::fs;

pub async fn battery<'a>(p: Option<String>) -> Option<String> {
    let bat: i32 = match p {
        Some(p) => p.trim().parse::<i32>().unwrap(),
        None => 0,
    };

    let perc = fs::read_to_string(format!(
        "/sys/class/power_supply/BAT{}/capacity",
        bat.to_owned()
    ))
    .await
    .ok()?
    .parse::<i32>()
    .ok();

    let stat = fs::read_to_string(format!("/sys/class/power_supply/BAT{}/status", bat))
        .await
        .ok()?;

    let icon = if stat == "Discharging" { "" } else { "" };

    let perc_total = match perc {
        Some(perc) => perc,
        None => { return None },
    };

    Some(format!("{} {}%", icon, perc_total))
}
