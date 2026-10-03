use std::sync::{Arc, Mutex};

use worth_runtime_world::facade::ProductUnpublishedRecoveryHandle;

use super::{
    WorthQueryUnpublishedIdempotencyEntry, WorthQueryUnpublishedIdempotencyKey,
    WorthQueryUnpublishedIdempotencyPosture as Posture, WorthQueryUnpublishedIdempotencyStore,
};
use crate::domain_computation::primary_graph::provider::ManagedUnpublishedAttempt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph::provider) enum ManagedUnpublishedRecoveryStop {
    Missing,
    Busy,
    Capacity,
    Admission(worth_relational::facade::mvcc::CompanionPreflightStop),
}

/// Checked-out custody of the exact World partial. Dropping a failed
/// pre-effect attempt restores the same provider entry without a new lookup.
pub(in crate::domain_computation::primary_graph::provider) struct ManagedUnpublishedRecoveryGuard {
    store: Arc<Mutex<WorthQueryUnpublishedIdempotencyStore>>,
    entry: Arc<Mutex<WorthQueryUnpublishedIdempotencyEntry>>,
    key: WorthQueryUnpublishedIdempotencyKey,
    handle: ProductUnpublishedRecoveryHandle,
    prepared_successor: Option<ProductUnpublishedRecoveryHandle>,
    attempt: Option<ManagedUnpublishedAttempt>,
    armed: bool,
}

impl ManagedUnpublishedRecoveryGuard {
    pub(super) fn begin(
        store: Arc<Mutex<WorthQueryUnpublishedIdempotencyStore>>,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<Self, ManagedUnpublishedRecoveryStop> {
        let owner = store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let key = owner
            .recovery_key(handle)
            .cloned()
            .ok_or(ManagedUnpublishedRecoveryStop::Missing)?;
        let entry = Arc::clone(
            owner
                .by_key
                .get(&key)
                .ok_or(ManagedUnpublishedRecoveryStop::Missing)?,
        );
        let mut locked = entry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if locked.posture != Posture::Retained || &locked.recovery_handle != handle {
            return Err(ManagedUnpublishedRecoveryStop::Busy);
        }
        let attempt = locked
            .managed
            .take()
            .ok_or(ManagedUnpublishedRecoveryStop::Missing)?;
        locked.posture = Posture::Recovering;
        drop(locked);
        drop(owner);
        Ok(Self {
            store,
            entry,
            key,
            handle: handle.clone(),
            prepared_successor: None,
            attempt: Some(attempt),
            armed: true,
        })
    }

    pub(in crate::domain_computation::primary_graph::provider) fn handle(
        &self,
    ) -> &ProductUnpublishedRecoveryHandle {
        &self.handle
    }

    pub(in crate::domain_computation::primary_graph::provider) fn attempt_mut(
        &mut self,
    ) -> &mut ManagedUnpublishedAttempt {
        self.attempt
            .as_mut()
            .expect("checked-out recovery retains its attempt")
    }

    pub(in crate::domain_computation::primary_graph::provider) fn take_attempt(
        &mut self,
    ) -> ManagedUnpublishedAttempt {
        self.attempt
            .take()
            .expect("performed recovery consumes its exact attempt")
    }

    /// Reserve the owner-issued successor handle before World can perform.
    /// The staged index entry is invisible while this checkout retains the
    /// original handle and the provider entry remains Recovering.
    pub(in crate::domain_computation::primary_graph::provider) fn reserve_successor(
        &mut self,
        successor: &ProductUnpublishedRecoveryHandle,
    ) -> Result<(), ManagedUnpublishedRecoveryStop> {
        assert!(self.prepared_successor.is_none());
        let admission = self
            .attempt
            .as_mut()
            .expect("checked-out recovery retains its attempt")
            .publication_admission_mut();
        let mut store = self
            .store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let maximum = store
            .maximum_recovery_mappings()
            .ok_or(ManagedUnpublishedRecoveryStop::Capacity)?;
        // The selected scan, possible relocation of initialized Vec slots,
        // and both terminal removals are all bounded before World executes.
        let relocated = if store.by_recovery.len() == store.by_recovery.capacity() {
            store.by_recovery.len()
        } else {
            0
        };
        let scans = maximum
            .checked_mul(2)
            .and_then(|terminal| terminal.checked_add(store.by_recovery.len()))
            .and_then(|work| work.checked_add(relocated))
            // finish performs both the selected provider-key lookup and its
            // removal. Bound those paths at the installed entry ceiling,
            // since unrelated reservations may arrive while World executes.
            .and_then(|work| {
                let path = provider_key_navigation_work(maximum)?;
                let name_bytes = self.key.0.comparison_name_len();
                let compared = path.checked_mul(name_bytes)?;
                work.checked_add(path.checked_add(compared)?.checked_mul(2)?)
            })
            .and_then(|work| work.checked_add(2))
            .ok_or(ManagedUnpublishedRecoveryStop::Capacity)?;
        admission
            .charge_external_work(
                u64::try_from(scans).map_err(|_| ManagedUnpublishedRecoveryStop::Capacity)?,
            )
            .map_err(ManagedUnpublishedRecoveryStop::Admission)?;
        if successor == &self.handle {
            return Ok(());
        }
        if store.recovery_key(successor).is_some() {
            return Err(ManagedUnpublishedRecoveryStop::Busy);
        }
        if store.by_recovery.len() >= maximum {
            return Err(ManagedUnpublishedRecoveryStop::Capacity);
        }
        if store.by_recovery.len() == store.by_recovery.capacity() {
            let bytes = store
                .by_recovery
                .len()
                .checked_add(1)
                .ok_or(ManagedUnpublishedRecoveryStop::Capacity)?
                .checked_mul(std::mem::size_of::<(
                    ProductUnpublishedRecoveryHandle,
                    WorthQueryUnpublishedIdempotencyKey,
                )>())
                .ok_or(ManagedUnpublishedRecoveryStop::Capacity)?;
            admission
                .admit_read_scratch(
                    u64::try_from(bytes).map_err(|_| ManagedUnpublishedRecoveryStop::Capacity)?,
                )
                .map_err(ManagedUnpublishedRecoveryStop::Admission)?;
        }
        store
            .by_recovery
            .try_reserve_exact(1)
            .map_err(|_| ManagedUnpublishedRecoveryStop::Capacity)?;
        store
            .by_recovery
            .push((successor.clone(), self.key.clone()));
        self.prepared_successor = Some(successor.clone());
        Ok(())
    }

    /// World returned another partial under the preissued successor handle.
    /// Rebind the existing provider row without allocating or admitting after
    /// the owner effect. The old World record is released by the caller.
    pub(in crate::domain_computation::primary_graph::provider) fn retain_successor(
        mut self,
        successor: &ProductUnpublishedRecoveryHandle,
    ) {
        assert!(self.attempt.is_some());
        assert!(
            successor == &self.handle || self.prepared_successor.as_ref() == Some(successor),
            "World returned a recovery handle that was not reserved before effects"
        );
        let mut store = self
            .store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut entry = self
            .entry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert_eq!(entry.recovery_handle, self.handle);
        assert!(matches!(entry.posture, Posture::Recovering));
        entry.recovery_handle = successor.clone();
        assert!(entry
            .managed
            .replace(self.attempt.take().unwrap())
            .is_none());
        entry.posture = Posture::Retained;
        if successor != &self.handle {
            store.remove_recovery(&self.handle);
            self.prepared_successor = None;
        }
        self.armed = false;
    }

    pub(in crate::domain_computation::primary_graph::provider) fn finish(mut self) {
        assert!(
            self.attempt.is_none(),
            "only a transferred recovery can finish"
        );
        let mut store = self
            .store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(successor) = self.prepared_successor.take() {
            store.remove_recovery(&successor);
        }
        let removed = store.release_exact(&self.key, &self.handle);
        self.armed = false;
        drop(removed);
    }
}

/// A B-tree root can contain one key; every deeper level needs at least
/// five keys per non-root node. One selected lookup compares at most eleven
/// keys per possible level, and never compares the same stored key twice.
fn provider_key_navigation_work(entries: usize) -> Option<usize> {
    let mut minimum = 1usize;
    let mut levels = 1usize;
    while minimum < entries {
        minimum = minimum.checked_mul(6)?.checked_add(5)?;
        levels = levels.checked_add(1)?;
    }
    entries.min(levels.checked_mul(11)?).checked_add(1)
}

impl Drop for ManagedUnpublishedRecoveryGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let mut store = self
            .store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut entry = self
            .entry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(matches!(entry.posture, Posture::Recovering));
        assert_eq!(entry.recovery_handle, self.handle);
        if let Some(attempt) = self.attempt.take() {
            assert!(entry.managed.replace(attempt).is_none());
        }
        entry.posture = Posture::Retained;
        if let Some(successor) = self.prepared_successor.take() {
            store.remove_recovery(&successor);
        }
    }
}
