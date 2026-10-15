pub fn format_time(time: f64) -> String {
    if time.is_nan() {
        return "00:00".into();
    }
    let t = time as u64;
    let h = t / 3600;
    let m = (t % 3600) / 60;
    let s = t % 60;
    if h > 0 {
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", m, s)
    }
}
