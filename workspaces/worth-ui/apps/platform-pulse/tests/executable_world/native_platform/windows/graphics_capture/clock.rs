//! Convert raw desktop presentation and input QPC into one checked100ns basis.
use super::{capture_failure, NativePlatformFailure};

pub(in super::super) fn qpc_100ns() -> Result<i64, NativePlatformFailure> {
    let frequency = winsafe::QueryPerformanceFrequency().map_err(capture_failure)?;
    let counter = winsafe::QueryPerformanceCounter().map_err(capture_failure)?;
    convert_counter(counter, frequency).ok_or_else(|| capture_failure("invalid QPC conversion"))
}

pub(super) fn convert_counter(counter: i64, frequency: i64) -> Option<i64> {
    if counter < 0 || frequency <= 0 {
        return None;
    }
    i64::try_from(i128::from(counter).checked_mul(10_000_000)? / i128::from(frequency)).ok()
}

pub(super) fn validate_timestamp(previous: Option<i64>, captured: i64, callback: i64) -> bool {
    captured >= 0 && captured <= callback && previous.is_none_or(|prior| captured > prior)
}
