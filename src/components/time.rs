use chrono;

pub async fn time() -> Option<String> {
    let now = chrono::Local::now();
    Some(now.format(" %Y-%m-%d %H:%M:%S").to_string())
}
