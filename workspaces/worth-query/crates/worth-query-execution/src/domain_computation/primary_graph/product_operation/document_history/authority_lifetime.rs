//! Revocable lifetime of one selected document slot or one output callback.
use std::sync::{Arc, Mutex};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_relational::facade::transactions::RelationalExecutionDenialCause;

struct LifetimeBacking {
    live: Mutex<bool>,
    _memory: Option<ExecutionMemoryReservation>,
}

/// This owner alone can end a lifetime. It carries no permission or head data.
pub(in crate::domain_computation::primary_graph) struct OwnedHistoryAuthorityLifetime {
    backing: Arc<LifetimeBacking>,
}

/// A readonly dependency on the original owner, never a permission result.
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct HistoryAuthorityLifetime {
    backing: Arc<LifetimeBacking>,
}

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct HistoryAuthorityExpired;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum HistoryAuthorityTryStop {
    Busy,
    Expired,
}

impl OwnedHistoryAuthorityLifetime {
    pub(in crate::domain_computation::primary_graph) fn reserve(
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalExecutionDenialCause> {
        let memory = super::allocation::reserve_arc::<LifetimeBacking>(execution)?;
        Ok(Self {
            backing: Arc::new(LifetimeBacking {
                live: Mutex::new(true),
                _memory: memory,
            }),
        })
    }

    pub(in crate::domain_computation::primary_graph) fn retain(&self) -> HistoryAuthorityLifetime {
        HistoryAuthorityLifetime {
            backing: Arc::clone(&self.backing),
        }
    }
}

impl HistoryAuthorityLifetime {
    /// Native-borrowed readers refuse contention instead of inverting owner order.
    pub(in crate::domain_computation::primary_graph) fn try_while_live<T>(
        &self,
        handoff: impl FnOnce() -> T,
    ) -> Result<T, HistoryAuthorityTryStop> {
        let live = self.backing.live.try_lock().map_err(|stop| match stop {
            std::sync::TryLockError::WouldBlock => HistoryAuthorityTryStop::Busy,
            std::sync::TryLockError::Poisoned(_) => HistoryAuthorityTryStop::Expired,
        })?;
        if !*live {
            return Err(HistoryAuthorityTryStop::Expired);
        }
        Ok(handoff())
    }

    /// Keep only the short original-owner handoff under this seal. Do not
    /// reacquire the selection/core or execute application/producer code here.
    pub(in crate::domain_computation::primary_graph) fn while_live<T>(
        &self,
        handoff: impl FnOnce() -> T,
    ) -> Result<T, HistoryAuthorityExpired> {
        let live = self
            .backing
            .live
            .lock()
            .map_err(|_| HistoryAuthorityExpired)?;
        if !*live {
            return Err(HistoryAuthorityExpired);
        }
        Ok(handoff())
    }
}

impl Drop for OwnedHistoryAuthorityLifetime {
    fn drop(&mut self) {
        *self
            .backing
            .live
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::sync_channel;
    use std::time::Duration;

    fn lifetime() -> OwnedHistoryAuthorityLifetime {
        OwnedHistoryAuthorityLifetime {
            backing: Arc::new(LifetimeBacking {
                live: Mutex::new(true),
                _memory: None,
            }),
        }
    }

    #[test]
    fn borrowed_handoff_refuses_contention_and_retries_the_same_live_owner() {
        let owner = lifetime();
        let stamp = owner.retain();
        let held = owner.backing.live.lock().unwrap();
        let handed_off = std::cell::Cell::new(false);
        assert_eq!(
            stamp.try_while_live(|| handed_off.set(true)),
            Err(HistoryAuthorityTryStop::Busy)
        );
        assert!(!handed_off.get());
        drop(held);
        stamp.try_while_live(|| handed_off.set(true)).unwrap();
        assert!(handed_off.get());
        drop(owner);
        assert_eq!(
            stamp.try_while_live(|| panic!("ended owner handed off")),
            Err(HistoryAuthorityTryStop::Expired)
        );
    }

    #[test]
    fn ending_original_owner_refuses_every_retained_handoff() {
        let owner = lifetime();
        let first = owner.retain();
        let second = first.clone();
        assert_eq!(first.while_live(|| 7).unwrap(), 7);
        drop(owner);
        assert!(first.while_live(|| panic!("expired handoff ran")).is_err());
        assert!(second
            .while_live(|| panic!("cloned expired handoff ran"))
            .is_err());
    }

    #[test]
    fn owner_cannot_end_while_original_handoff_holds_its_seal() {
        let owner = lifetime();
        let stamp = owner.retain();
        let later = stamp.clone();
        let (entered_tx, entered_rx) = sync_channel(0);
        let (release_tx, release_rx) = sync_channel(0);
        let reader = std::thread::spawn(move || {
            stamp.while_live(|| {
                entered_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            })
        });
        entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        // The lock itself, not a timing comparison, proves the seal is held.
        assert!(owner.backing.live.try_lock().is_err());
        let (ending_tx, ending_rx) = sync_channel(0);
        let closer = std::thread::spawn(move || {
            ending_tx.send(()).unwrap();
            drop(owner);
        });
        ending_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        release_tx.send(()).unwrap();
        reader.join().unwrap().unwrap();
        closer.join().unwrap();
        assert!(later
            .while_live(|| panic!("released owner reopened"))
            .is_err());
    }

    #[test]
    fn panic_never_reopens_a_poisoned_handoff() {
        let owner = lifetime();
        let stamp = owner.retain();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = stamp.while_live(|| panic!("handoff failed"));
        }))
        .is_err());
        assert!(stamp.while_live(|| panic!("poisoned handoff ran")).is_err());
        drop(owner);
        assert!(stamp
            .while_live(|| panic!("closed poisoned handoff ran"))
            .is_err());
    }
}
