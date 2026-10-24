use std::sync::Arc;
use std::sync::Mutex;

use worth_runtime_world::facade::{
    ProductUnpublishedRecoveryHandle, RuntimeWorldRecoveryDenial, RuntimeWorldRecoveryPort,
};

use super::WorthQueryUnpublishedIdempotencyEntry;
use super::WorthQueryUnpublishedIdempotencyKey;
use super::WorthQueryUnpublishedIdempotencyPosture as Posture;
use super::WorthQueryUnpublishedIdempotencyStore;
use crate::domain_computation::primary_graph::provider::ManagedUnpublishedAttempt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph::provider) enum ManagedUnpublishedRecoveryStop {
    Missing,
    Busy,
    Capacity,
}

/// Checked-out custody of the exact World partial. Dropping a failed
/// pre-effect attempt restores the same provider entry without a new lookup.
pub(in crate::domain_computation::primary_graph::provider) struct ManagedUnpublishedRecoveryGuard {
    store: Arc<Mutex<WorthQueryUnpublishedIdempotencyStore>>,
    entry: Arc<Mutex<WorthQueryUnpublishedIdempotencyEntry>>,
    key: WorthQueryUnpublishedIdempotencyKey,
    handle: ProductUnpublishedRecoveryHandle,
    recovery: RuntimeWorldRecoveryPort,
    prepared_successor: Option<ProductUnpublishedRecoveryHandle>,
    attempt: Option<ManagedUnpublishedAttempt>,
    armed: bool,
}

impl ManagedUnpublishedRecoveryGuard {
    pub(super) fn begin(
        store: Arc<Mutex<WorthQueryUnpublishedIdempotencyStore>>,
        handle: &ProductUnpublishedRecoveryHandle,
        recovery: RuntimeWorldRecoveryPort,
    ) -> Result<Self, ManagedUnpublishedRecoveryStop> {
        Self::reconcile_retained_aliases(&store, handle, &recovery)?;
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
            recovery,
            prepared_successor: None,
            attempt: Some(attempt),
            armed: true,
        })
    }

    /// An unavailable native inspection leaves both pre-reserved mappings intact.
    /// A later admission resolves them from owner evidence before checking out custody.
    fn reconcile_retained_aliases(
        store: &Arc<Mutex<WorthQueryUnpublishedIdempotencyStore>>,
        requested: &ProductUnpublishedRecoveryHandle,
        recovery: &RuntimeWorldRecoveryPort,
    ) -> Result<(), ManagedUnpublishedRecoveryStop> {
        use ManagedUnpublishedRecoveryStop as Stop;
        let (key, entry, preferred, alternate) = {
            let owner = store
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let key = owner
                .recovery_key(requested)
                .cloned()
                .ok_or(Stop::Missing)?;
            let entry = Arc::clone(owner.by_key.get(&key).ok_or(Stop::Missing)?);
            let locked = entry
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if locked.posture != Posture::Retained {
                return Err(Stop::Busy);
            }
            let preferred = locked.recovery_handle.clone();
            let alternate = owner
                .by_recovery
                .iter()
                .find(|(handle, candidate)| candidate == &key && handle != &preferred)
                .map(|(handle, _)| handle.clone());
            drop(locked);
            (key, entry, preferred, alternate)
        };
        let Some(alternate) = alternate else {
            return Ok(());
        };
        // World can materialize abandoned custody here. No Query lock spans inspection.
        let resolved = match recovery.inspect_effects(&preferred).map(|_| ()) {
            Ok(()) => preferred.clone(),
            Err(RuntimeWorldRecoveryDenial::MissingRecord) => {
                match recovery.inspect_effects(&alternate).map(|_| ()) {
                    Ok(()) => alternate,
                    Err(RuntimeWorldRecoveryDenial::MissingRecord) => return Err(Stop::Missing),
                    Err(_) => return Err(Stop::Busy),
                }
            }
            Err(_) => return Err(Stop::Busy),
        };
        let mut owner = store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !owner
            .by_key
            .get(&key)
            .is_some_and(|current| Arc::ptr_eq(current, &entry))
            || owner.recovery_key(&resolved) != Some(&key)
        {
            return Err(Stop::Busy);
        }
        let mut locked = entry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if locked.posture != Posture::Retained || locked.recovery_handle != preferred {
            return Err(Stop::Busy);
        }
        locked.recovery_handle = resolved.clone();
        owner
            .by_recovery
            .retain(|(handle, candidate)| candidate != &key || handle == &resolved);
        if requested != &resolved {
            return Err(Stop::Missing);
        }
        Ok(())
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
        let mut store = self
            .store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if successor == &self.handle {
            return Ok(());
        }
        if store.recovery_key(successor).is_some() {
            return Err(ManagedUnpublishedRecoveryStop::Busy);
        }
        if let Some(maximum) = store.maximum_entries {
            let maximum = maximum
                .checked_mul(2)
                .ok_or(ManagedUnpublishedRecoveryStop::Capacity)?;
            if store.by_recovery.len() >= maximum {
                return Err(ManagedUnpublishedRecoveryStop::Capacity);
            }
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

impl Drop for ManagedUnpublishedRecoveryGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        // Native attempt Drop has already retained any actual successor effects.
        // Only authoritative absence permits discarding its reserved mapping.
        let successor_evidence = self
            .prepared_successor
            .as_ref()
            .map(|handle| self.recovery.inspect_effects(handle).map(|_| ()));
        let mut store = self
            .store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut entry = self
            .entry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(attempt) = self.attempt.take() {
            entry.managed = Some(attempt);
        }
        entry.posture = Posture::Retained;
        if let Some(successor) = self.prepared_successor.take() {
            match successor_evidence {
                Some(Ok(())) => {
                    entry.recovery_handle = successor;
                    store.remove_recovery(&self.handle);
                }
                Some(Err(RuntimeWorldRecoveryDenial::MissingRecord)) => {
                    store.remove_recovery(&successor);
                }
                Some(Err(_)) => {
                    // Both mappings were reserved before effects. Keep both until
                    // a fresh admission can resolve the actual native record.
                    entry.recovery_handle = successor;
                }
                None => {}
            }
        }
    }
}
