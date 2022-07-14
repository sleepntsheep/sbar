use chrono;

pub async fn time(args: &Vec<String>) -> Option<String> {
    let mut fmtstr = " %Y-%m-%d %H:%M:%S";
    if args.len() >= 1 {
        fmtstr = args.first()?;
    }
    let now = chrono::Local::now();
    Some(now.format(fmtstr).to_string())
}
