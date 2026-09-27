//! The counter a resize trace shares with an outside capture.
//!
//! On Windows this is the performance counter, which every process reads on
//! one basis. Elsewhere no shared counter is qualified, so nothing is traced.

#[cfg(windows)]
pub(super) fn frequency() -> Option<i64> {
    winsafe::QueryPerformanceFrequency()
        .ok()
        .filter(|frequency| *frequency > 0)
}

#[cfg(windows)]
pub(super) fn counter() -> Option<i64> {
    winsafe::QueryPerformanceCounter().ok()
}

#[cfg(not(windows))]
pub(super) fn frequency() -> Option<i64> {
    None
}

#[cfg(not(windows))]
pub(super) fn counter() -> Option<i64> {
    None
}
