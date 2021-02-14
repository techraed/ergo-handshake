use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn make_timestamp() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("internal error: current time is before unix epoch");
    now.as_millis() as u64
}