//! Latch of one caller-owned patch-position pause.
use crate::execution::monotonic_time::RelationalCutoverDeadline;

/// Release gate for the reservation-held pause. Opening never blocks, so an
/// observer can free the publisher before it asserts, and waiting is bounded, so
/// a seam that is moved out of the reservation window fails a run by name
/// instead of hanging it.
#[derive(Debug, Default)]
pub struct PatchPositionReservationLatch {
    opened: std::sync::Mutex<bool>,
    signal: std::sync::Condvar,
}

impl PatchPositionReservationLatch {
    // The entire protected state is one bool. Either value remains a valid
    // latch state after unwinding; no observer callback runs under this lock.
    pub fn open(&self) {
        let mut opened = self
            .opened
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        *opened = true;
        self.signal.notify_all();
    }

    pub(super) fn wait_for_open(&self, budget: std::time::Duration) {
        let deadline = RelationalCutoverDeadline::system_after(budget);
        let mut opened = self
            .opened
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        while !*opened {
            let Some(remaining) = deadline.remaining() else {
                return;
            };
            opened = self
                .signal
                .wait_timeout(opened, remaining)
                .unwrap_or_else(|error| error.into_inner())
                .0;
        }
    }
}
