//! One borrowed installed call on the existing owner close-admission ledger.
use super::attempt::RuntimeWorldOperationLedgerState;
use super::RuntimeWorldOwnerRoot;
use crate::lifecycle::{close::RuntimeWorldCloseState, owner::RuntimeWorldBootstrapState};
use std::sync::{Mutex, MutexGuard};

/// Holds no publication attempt, copied owner or separately allocated state.
#[must_use = "the whole installed call must retain its actual owner admission"]
pub(in crate::lifecycle) struct RuntimeWorldCallAdmission<'owner> {
    ledger: &'owner Mutex<RuntimeWorldOperationLedgerState>,
}

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(in crate::lifecycle) fn try_admit_call(&self) -> Option<RuntimeWorldCallAdmission<'_>> {
        let _ordinary_admission = self.state.operation.call_exclusion.try_read().ok()?;
        // D9 admits concurrent ordinary calls; only branch commitment serializes
        // them. Use the same short metadata lock order as operation reservation
        // and final close admission: exclusion, bootstrap, operation, close. No owner work,
        // callback or I/O occurs under these guards.
        let bootstrap = owner_metadata(&self.state.bootstrap);
        if !bootstrap_ready(&bootstrap) {
            return None;
        }
        let mut ledger = owner_metadata(&self.state.operation.state);
        let close = owner_metadata(&self.state.close);
        RuntimeWorldCallAdmission::admit(
            &self.state.operation.state,
            &mut ledger,
            close.state(),
            self.owner_is_present(),
        )
    }
}

fn bootstrap_ready(bootstrap: &RuntimeWorldBootstrapState) -> bool {
    match bootstrap {
        RuntimeWorldBootstrapState::Performed => true,
        RuntimeWorldBootstrapState::Unperformed
        | RuntimeWorldBootstrapState::InProgress
        | RuntimeWorldBootstrapState::Pending(_) => false,
    }
}
impl<'owner> RuntimeWorldCallAdmission<'owner> {
    fn admit(
        ledger: &'owner Mutex<RuntimeWorldOperationLedgerState>,
        state: &mut RuntimeWorldOperationLedgerState,
        close: RuntimeWorldCloseState,
        present: bool,
    ) -> Option<Self> {
        match close {
            RuntimeWorldCloseState::Open => {}
            RuntimeWorldCloseState::Closing | RuntimeWorldCloseState::Closed => return None,
        }
        if !present {
            return None;
        }
        state.active = state.active.checked_add(1)?;
        Some(Self { ledger })
    }
}

impl Drop for RuntimeWorldCallAdmission<'_> {
    fn drop(&mut self) {
        // Match the existing operation-reservation release contract. A failed
        // try-lock must never lose the original active entry during unwinding.
        let mut ledger = self
            .ledger
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        ledger.active = ledger
            .active
            .checked_sub(1)
            .expect("an admitted installed call owns one operation entry");
    }
}

fn owner_metadata<T>(metadata: &Mutex<T>) -> MutexGuard<'_, T> {
    metadata.lock().unwrap_or_else(|error| error.into_inner())
}

#[cfg(test)]
mod tests {
    use super::super::attempt::RuntimeWorldOperationLedger;
    use super::*;
    #[test]
    fn concurrent_installed_calls_all_retain_admission_until_their_own_return() {
        let ledger = RuntimeWorldOperationLedger::new();
        let start = std::sync::Barrier::new(32);
        let finish = std::sync::Barrier::new(33);
        let (send, receive) = std::sync::mpsc::channel();
        std::thread::scope(|scope| {
            for _ in 0..32 {
                let (ledger, start, finish, send) = (&ledger, &start, &finish, send.clone());
                scope.spawn(move || {
                    start.wait();
                    let admission = {
                        let mut state = ledger.state.lock().unwrap();
                        RuntimeWorldCallAdmission::admit(
                            &ledger.state,
                            &mut state,
                            RuntimeWorldCloseState::Open,
                            true,
                        )
                    };
                    send.send(admission.is_some()).unwrap();
                    finish.wait();
                    drop(admission);
                });
            }
            let admitted = (0..32).map(|_| receive.recv().unwrap()).collect::<Vec<_>>();
            let active = ledger.active();
            finish.wait();
            assert!(admitted.into_iter().all(|admitted| admitted));
            assert_eq!(active, 32);
        });
        assert_eq!(ledger.active(), 0);
    }
    #[test]
    fn ordinary_call_admission_preserves_bootstrap_and_closing_exclusions() {
        let ledger = RuntimeWorldOperationLedger::new();
        for bootstrap in [
            RuntimeWorldBootstrapState::Unperformed,
            RuntimeWorldBootstrapState::InProgress,
        ] {
            assert!(!bootstrap_ready(&bootstrap));
            assert_eq!(ledger.active(), 0);
        }
        for close in [
            RuntimeWorldCloseState::Closing,
            RuntimeWorldCloseState::Closed,
        ] {
            let mut state = ledger.state.lock().unwrap();
            assert!(
                RuntimeWorldCallAdmission::admit(&ledger.state, &mut state, close, true).is_none()
            );
            assert_eq!(state.active, 0);
        }
        let mut state = ledger.state.lock().unwrap();
        assert!(RuntimeWorldCallAdmission::admit(
            &ledger.state,
            &mut state,
            RuntimeWorldCloseState::Open,
            false
        )
        .is_none());
        assert_eq!(state.active, 0);
    }
    #[test]
    fn call_admission_releases_its_exact_entry_after_return_and_unwind() {
        let ledger = RuntimeWorldOperationLedger::new();
        let admit = || {
            let mut state = ledger.state.lock().unwrap();
            RuntimeWorldCallAdmission::admit(
                &ledger.state,
                &mut state,
                RuntimeWorldCloseState::Open,
                true,
            )
            .unwrap()
        };
        assert_eq!(ledger.active(), 0);
        let admission = admit();
        assert_eq!(ledger.active(), 1);
        drop(admission);
        assert_eq!(ledger.active(), 0);
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _admission = admit();
            assert_eq!(ledger.active(), 1);
            panic!("original installed-call unwind");
        }));
        assert!(panic.is_err());
        assert_eq!(ledger.active(), 0);
        let admission = admit();
        assert_eq!(ledger.active(), 1);
        drop(admission);
        assert_eq!(ledger.active(), 0);
    }
}
