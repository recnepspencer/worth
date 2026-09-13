//! Feature-only scheduling after a real provider attempt registration.

use std::num::NonZeroUsize;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

#[derive(Clone, Default)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryApplicationAttemptOperationControl
{
    pending: Arc<Mutex<Option<Arc<ExactRegistrationPause>>>>,
}

struct ExactRegistrationPause {
    target: usize,
    state: Mutex<(usize, bool)>,
    changed: Condvar,
}

pub struct WorthQueryApplicationAttemptRegistrationPause {
    control: WorthQueryApplicationAttemptOperationControl,
    latch: Arc<ExactRegistrationPause>,
}

impl WorthQueryApplicationAttemptOperationControl {
    pub(super) fn pause_after_registration(
        &self,
        attempts: NonZeroUsize,
    ) -> WorthQueryApplicationAttemptRegistrationPause {
        let latch = Arc::new(ExactRegistrationPause {
            target: attempts.get(),
            state: Mutex::new((0, false)),
            changed: Condvar::new(),
        });
        let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        assert!(pending.is_none(), "one registration pause may be armed");
        *pending = Some(Arc::clone(&latch));
        WorthQueryApplicationAttemptRegistrationPause {
            control: self.clone(),
            latch,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn after_registration(&self) {
        let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        let Some(latch) = pending.as_ref().map(Arc::clone) else {
            return;
        };
        let mut state = latch.state.lock().unwrap_or_else(|e| e.into_inner());
        state.0 += 1;
        if state.0 == latch.target {
            pending.take();
        }
        drop(pending);
        latch.changed.notify_all();
        let (state, _) = latch
            .changed
            .wait_timeout_while(state, Duration::from_secs(30), |state| !state.1)
            .unwrap_or_else(|e| e.into_inner());
        assert!(
            state.1,
            "application registration pause exceeded release budget"
        );
    }

    fn clear(&self, latch: &Arc<ExactRegistrationPause>) {
        let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        if pending
            .as_ref()
            .is_some_and(|armed| Arc::ptr_eq(armed, latch))
        {
            pending.take();
        }
    }
}

impl WorthQueryApplicationAttemptRegistrationPause {
    pub fn wait_until_reached(&self, timeout: Duration) -> bool {
        let state = self.latch.state.lock().unwrap_or_else(|e| e.into_inner());
        self.latch
            .changed
            .wait_timeout_while(state, timeout, |state| state.0 != self.latch.target)
            .unwrap_or_else(|e| e.into_inner())
            .0
             .0
            == self.latch.target
    }

    pub fn release(&self) {
        self.control.clear(&self.latch);
        self.latch.state.lock().unwrap_or_else(|e| e.into_inner()).1 = true;
        self.latch.changed.notify_all();
    }
}

impl Drop for WorthQueryApplicationAttemptRegistrationPause {
    fn drop(&mut self) {
        self.release();
    }
}
