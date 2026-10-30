use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
};

use worth_signal::facade::RawCompletionEnvelope;

use crate::physical_runtime::{
    PhysicalSignalAspectBindingDigest, PhysicalSignalSettlementOutcome, PhysicalWorkIdentity,
};

use super::PhysicalWorkSignalOwner;

pub(super) struct PhysicalSignalReconciliation {
    capacity: usize,
    pending: Mutex<VecDeque<PendingPhysicalCompletion>>,
    overflow: AtomicU64,
}

struct PendingPhysicalCompletion {
    identity: PhysicalWorkIdentity,
    route: PhysicalSignalAspectBindingDigest,
    envelope: RawCompletionEnvelope,
}

impl PhysicalSignalReconciliation {
    pub(super) fn bounded(capacity: usize) -> Self {
        Self {
            capacity,
            pending: Mutex::new(VecDeque::with_capacity(capacity)),
            overflow: AtomicU64::new(0),
        }
    }

    pub(super) fn retain(
        &self,
        identity: PhysicalWorkIdentity,
        route: PhysicalSignalAspectBindingDigest,
        envelope: RawCompletionEnvelope,
    ) -> bool {
        let mut pending = self
            .pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if pending.iter().any(|entry| entry.identity == identity) {
            return true;
        }
        if pending.len() == self.capacity {
            let _ = {
                let update_counter = &self.overflow;
                let mut value = update_counter.load(Ordering::Relaxed);
                loop {
                    let next_value = Some(value.saturating_add(1));
                    let Some(updated_value) = next_value else {
                        break Err(value);
                    };
                    match update_counter.compare_exchange_weak(
                        value,
                        updated_value,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    ) {
                        Ok(previous) => break Ok(previous),
                        Err(reloaded_value) => value = reloaded_value,
                    }
                }
            };
            return false;
        }
        pending.push_back(PendingPhysicalCompletion {
            identity,
            route,
            envelope,
        });
        true
    }

    pub(super) fn resolve(&self, identity: PhysicalWorkIdentity) {
        let mut pending = self
            .pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(index) = pending.iter().position(|entry| entry.identity == identity) {
            pending.remove(index);
        }
    }

    pub(super) fn reconcile(
        &self,
        owner: &PhysicalWorkSignalOwner,
    ) -> Vec<(PhysicalWorkIdentity, PhysicalSignalSettlementOutcome)> {
        let mut resolved = Vec::new();
        if owner.require_available().is_err() {
            return resolved;
        }
        loop {
            let pending = self
                .pending
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .pop_front();
            let Some(pending) = pending else {
                return resolved;
            };
            let outcome = owner.route(pending.route).map_or(
                PhysicalSignalSettlementOutcome::DerivedStateUnavailable,
                |route| route.record_settlement(pending.envelope.clone()),
            );
            if outcome == PhysicalSignalSettlementOutcome::DerivedStateUnavailable {
                self.pending
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .push_front(pending);
                return resolved;
            }
            resolved.push((pending.identity, outcome));
        }
    }

    pub(super) fn counts(&self) -> (usize, u64) {
        (
            self.pending
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .len(),
            self.overflow.load(Ordering::Acquire),
        )
    }
}
