use chrono::Local;

pub fn get_time_stamp() -> String {
    let now = Local::now();
    let formatted_time = now.format("%Y-%m-%d %H:%M:%S (%A)").to_string();
    formatted_time
}
