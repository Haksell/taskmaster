use std::time::{Duration, SystemTime};

pub fn is_time_elapsed(started_at: SystemTime, duration: u64) -> bool {
    let current_time = SystemTime::now();
    let elapsed_time = current_time
        .duration_since(started_at)
        .unwrap_or(Duration::from_secs(0));
    elapsed_time.as_secs() >= duration
}
