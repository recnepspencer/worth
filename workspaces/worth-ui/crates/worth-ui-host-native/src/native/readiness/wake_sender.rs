//! How readiness wakes reach the thread that runs the host: posted to the
//! platform event loop, or queued for the offscreen pump. Either sender is
//! cloned into worker threads; either reports a closed loop the same way.

use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use winit::event_loop::EventLoopProxy;

use super::UiNativeApplicationWake;

#[derive(Clone)]
pub(crate) enum UiNativeWakeSender {
    Platform(EventLoopProxy<UiNativeApplicationWake>),
    Offscreen(Arc<UiNativeOffscreenWakes>),
}

/// The offscreen pump's wake queue: the wakes posted and not yet dispatched,
/// and whether the pump still accepts them.
#[derive(Default)]
pub(crate) struct UiNativeOffscreenWakes {
    state: Mutex<UiNativeOffscreenWakeState>,
    posted: Condvar,
}

#[derive(Default)]
struct UiNativeOffscreenWakeState {
    pending: u64,
    closed: bool,
}

impl UiNativeWakeSender {
    /// Posts one wake; fails only when the loop no longer runs.
    pub(crate) fn send(&self) -> Result<(), ()> {
        match self {
            Self::Platform(proxy) => proxy.send_event(UiNativeApplicationWake).map_err(drop),
            Self::Offscreen(wakes) => wakes.post(),
        }
    }
}

impl UiNativeOffscreenWakes {
    fn post(&self) -> Result<(), ()> {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        if state.closed {
            return Err(());
        }
        state.pending += 1;
        self.posted.notify_all();
        Ok(())
    }

    /// Takes every wake posted so far, waiting until `deadline` for one when
    /// none is pending. `None` takes only what is already pending.
    pub(crate) fn take(&self, deadline: Option<Instant>) -> u64 {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(deadline) = deadline {
            state = self.wait_while_empty(state, deadline);
        }
        std::mem::take(&mut state.pending)
    }

    /// Waits until `deadline` for a wake without taking it; whether one is
    /// pending.
    pub(crate) fn await_posted(&self, deadline: Instant) -> bool {
        let state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        self.wait_while_empty(state, deadline).pending > 0
    }

    fn wait_while_empty<'state>(
        &self,
        mut state: MutexGuard<'state, UiNativeOffscreenWakeState>,
        deadline: Instant,
    ) -> MutexGuard<'state, UiNativeOffscreenWakeState> {
        while state.pending == 0 {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                break;
            };
            state = self
                .posted
                .wait_timeout(state, remaining)
                .unwrap_or_else(PoisonError::into_inner)
                .0;
        }
        state
    }

    /// Stops accepting wakes, as a platform loop does when it exits.
    pub(crate) fn close(&self) {
        self.state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .closed = true;
    }
}

#[cfg(test)]
#[path = "wake_sender_tests.rs"]
mod tests;
