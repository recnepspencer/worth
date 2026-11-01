//! One borrowed installed call on the existing owner close-admission ledger.
use super::attempt::RuntimeWorldOperationLedgerState;
use super::RuntimeWorldOwnerRoot;
use crate::lifecycle::{close::RuntimeWorldCloseState, owner::RuntimeWorldBootstrapState};
use std::sync::{Mutex, MutexGuard, TryLockError};

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
        let bootstrap = try_owner_metadata(&self.state.bootstrap)?;
        if !matches!(*bootstrap, RuntimeWorldBootstrapState::Performed) {
            return None;
        }
        let mut ledger = try_owner_metadata(&self.state.operation.state)?;
        let close = try_owner_metadata(&self.state.close)?;
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

fn try_owner_metadata<T>(metadata: &Mutex<T>) -> Option<MutexGuard<'_, T>> {
    match metadata.try_lock() {
        Ok(guard) => Some(guard),
        Err(TryLockError::Poisoned(error)) => Some(error.into_inner()),
        Err(TryLockError::WouldBlock) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::super::preparation_test_support;
    use crate::lifecycle::RuntimeWorldPublicationPort;
    use std::sync::Arc;

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
