use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::Result;

pub fn now() -> Result<Duration> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?)
}
