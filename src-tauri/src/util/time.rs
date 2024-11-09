use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::Up2bResult;

pub fn now() -> Up2bResult<Duration> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?)
}
