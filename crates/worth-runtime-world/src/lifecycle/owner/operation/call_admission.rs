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
        if !matches!(*bootstrap, RuntimeWorldBootstrapState::Performed) {
            return None;
        }
        let mut ledger = owner_metadata(&self.state.operation.state);
        let close = owner_metadata(&self.state.close);
        if !self.owner_is_present() || close.state() != RuntimeWorldCloseState::Open {
            return None;
        }
        ledger.active = ledger.active.checked_add(1)?;
        Some(RuntimeWorldCallAdmission {
            ledger: &self.state.operation.state,
        })
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
    use super::super::preparation_test_support;
    use crate::lifecycle::RuntimeWorldPublicationPort;
    use std::sync::Arc;

    #[test]
    fn public_call_ports_refuse_borrowed_preclose_and_serve_after_its_drop() {
        let (owner, _) = preparation_test_support::setup(1);
        let publication = RuntimeWorldPublicationPort::new(Arc::downgrade(&owner));
        let service: Arc<dyn crate::lifecycle::RuntimeWorldObservationService + Send + Sync> =
            owner.clone();
        let observation =
            crate::lifecycle::RuntimeWorldObservationPort::new(Arc::downgrade(&service));
        let policy = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            worth_execution::CancellationToken::new(),
            None,
        );
        let request = worth_execution::ExecutionRequest::serial(&policy);
        publication.with_execution_placement(|_| ()).unwrap();
        let admission = owner.try_prepare_close_admission(&request).unwrap();
        assert!(publication
            .with_execution_placement(|_| panic!("excluded call ran"))
            .is_err());
        assert!(observation.observe_root_product_branch().is_err());
        drop(admission);
        publication.with_execution_placement(|_| ()).unwrap();
    }

    #[test]
    fn concurrent_installed_calls_all_retain_admission_until_their_own_return() {
        let (owner, _) = preparation_test_support::setup(1);
        let start = std::sync::Barrier::new(32);
        let finish = std::sync::Barrier::new(33);
        let (send, receive) = std::sync::mpsc::channel();
        std::thread::scope(|scope| {
            for _ in 0..32 {
                let (owner, start, finish, send) = (&owner, &start, &finish, send.clone());
                scope.spawn(move || {
                    start.wait();
                    let admission = owner.try_admit_call();
                    send.send(admission.is_some()).unwrap();
                    finish.wait();
                    drop(admission);
                });
            }
            let admitted = (0..32).map(|_| receive.recv().unwrap()).collect::<Vec<_>>();
            let active = owner.state.operation.active();
            finish.wait();
            assert!(admitted.into_iter().all(|admitted| admitted));
            assert_eq!(active, 32);
        });
        assert_eq!(owner.state.operation.active(), 0);
    }

    #[test]
    fn ordinary_call_admission_preserves_bootstrap_and_closing_exclusions() {
        use crate::lifecycle::owner::RuntimeWorldBootstrapState;
        let (owner, _) = preparation_test_support::setup(1);
        for excluded in [
            RuntimeWorldBootstrapState::Unperformed,
            RuntimeWorldBootstrapState::InProgress,
        ] {
            *owner.state.bootstrap.lock().unwrap() = excluded;
            assert!(owner.try_admit_call().is_none());
            assert_eq!(owner.state.operation.active(), 0);
        }
        *owner.state.bootstrap.lock().unwrap() = RuntimeWorldBootstrapState::Performed;
        let request = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            worth_execution::CancellationToken::new(),
            None,
        );
        let execution = worth_execution::ExecutionRequest::serial(&request);
        owner
            .try_prepare_close_admission(&execution)
            .unwrap()
            .begin();
        assert!(owner.try_admit_call().is_none());
        assert_eq!(owner.state.operation.active(), 0);
    }

    #[test]
    fn placement_scope_releases_its_original_entry_after_return_and_unwind() {
        let (owner, _) = preparation_test_support::setup(1);
        let port = RuntimeWorldPublicationPort::new(Arc::downgrade(&owner));
        assert_eq!(owner.state.operation.active(), 0);
        port.with_execution_placement(|_| {
            assert_eq!(owner.state.operation.active(), 1);
        })
        .unwrap();
        assert_eq!(owner.state.operation.active(), 0);
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            port.with_execution_placement::<()>(|_| panic!("original installed-call unwind"))
        }));
        assert!(panic.is_err());
        assert_eq!(owner.state.operation.active(), 0);
        port.with_execution_placement(|_| {
            assert_eq!(owner.state.operation.active(), 1);
        })
        .unwrap();
        assert_eq!(owner.state.operation.active(), 0);
    }
}
