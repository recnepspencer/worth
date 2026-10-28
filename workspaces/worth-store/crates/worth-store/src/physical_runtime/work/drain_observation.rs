use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};

use super::{
    PhysicalWorkEffectFate, PhysicalWorkIdentity, PhysicalWorkRecoveryDisposition,
    PhysicalWorkTerminalStage,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalWorkDrainObservation {
    settled: Box<[PhysicalWorkIdentity]>,
    cancelled_before_dispatch: Box<[PhysicalWorkIdentity]>,
    continued_after_consumer_cancellation: Box<[PhysicalWorkIdentity]>,
    inspection_required: Box<[PhysicalWorkIdentity]>,
    released_before_dispatch: Box<[PhysicalWorkIdentity]>,
    residual: Box<[PhysicalWorkIdentity]>,
    derived_reconciliation_deferred: Box<[PhysicalWorkIdentity]>,
    evidence_capacity: usize,
    evidence_overflow: u64,
    safe_evidence_elided: u64,
}

pub(super) struct PhysicalWorkTerminalLedger {
    capacity: usize,
    entries: Mutex<Vec<PhysicalWorkTerminalEvent>>,
    evidence_overflow: AtomicU64,
    safe_evidence_elided: AtomicU64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PhysicalWorkTerminalEvent {
    Settled {
        identity: PhysicalWorkIdentity,
        fate: PhysicalWorkEffectFate,
        recovery: PhysicalWorkRecoveryDisposition,
        consumer_cancelled: bool,
    },
    CancelledBeforeDispatch(PhysicalWorkIdentity),
    ReleasedBeforeDispatch {
        identity: PhysicalWorkIdentity,
        consumer: Option<super::PhysicalWorkConsumerHandle>,
    },
    AbandonedAfterDispatch(PhysicalWorkIdentity),
    DerivedReconciliationDeferred(PhysicalWorkIdentity),
    DerivedReconciliationCompleted(PhysicalWorkIdentity),
}

impl PhysicalWorkTerminalLedger {
    pub(super) fn bounded(capacity: usize) -> Self {
        Self {
            capacity,
            entries: Mutex::new(Vec::with_capacity(capacity)),
            evidence_overflow: AtomicU64::new(0),
            safe_evidence_elided: AtomicU64::new(0),
        }
    }

    pub(super) fn record(&self, event: PhysicalWorkTerminalEvent) {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if entries.len() < self.capacity {
            entries.push(event);
        } else if event.must_retain() {
            if let PhysicalWorkTerminalEvent::DerivedReconciliationDeferred(identity) = event
                && let Some(same_work) = entries.iter().position(|retained| {
                    matches!(
                        retained,
                        PhysicalWorkTerminalEvent::Settled {
                            identity: settled,
                            ..
                        } if *settled == identity
                    )
                })
            {
                entries[same_work] = event;
                return;
            }
            if let Some(safe) = entries.iter().position(|retained| !retained.must_retain()) {
                entries.swap_remove(safe);
                entries.push(event);
                saturating_increment(&self.safe_evidence_elided);
            } else {
                saturating_increment(&self.evidence_overflow);
            }
        } else {
            saturating_increment(&self.safe_evidence_elided);
        }
    }

    pub(super) fn observe(
        &self,
        residual: impl IntoIterator<Item = (PhysicalWorkIdentity, PhysicalWorkTerminalStage)>,
    ) -> PhysicalWorkDrainObservation {
        let entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut settled = Vec::new();
        let mut cancelled_before_dispatch = Vec::new();
        let mut continued_after_consumer_cancellation = Vec::new();
        let mut inspection_required = Vec::new();
        let mut released_before_dispatch = Vec::new();
        let mut derived_reconciliation_deferred = Vec::new();
        for event in entries.iter().copied() {
            match event {
                PhysicalWorkTerminalEvent::Settled {
                    identity,
                    fate,
                    recovery,
                    consumer_cancelled: _,
                } if recovery == PhysicalWorkRecoveryDisposition::InspectionRequired
                    || matches!(
                        fate,
                        PhysicalWorkEffectFate::Indeterminate
                            | PhysicalWorkEffectFate::WrittenButSchedulerRejected
                            | PhysicalWorkEffectFate::StaleOrForeignOutcome
                    ) =>
                {
                    inspection_required.push(identity);
                }
                PhysicalWorkTerminalEvent::Settled {
                    identity,
                    consumer_cancelled: true,
                    ..
                } => continued_after_consumer_cancellation.push(identity),
                PhysicalWorkTerminalEvent::Settled { identity, .. } => settled.push(identity),
                PhysicalWorkTerminalEvent::CancelledBeforeDispatch(identity) => {
                    cancelled_before_dispatch.push(identity);
                }
                PhysicalWorkTerminalEvent::ReleasedBeforeDispatch { identity, .. } => {
                    released_before_dispatch.push(identity);
                }
                PhysicalWorkTerminalEvent::AbandonedAfterDispatch(identity) => {
                    inspection_required.push(identity);
                }
                PhysicalWorkTerminalEvent::DerivedReconciliationDeferred(identity) => {
                    settled.push(identity);
                    derived_reconciliation_deferred.push(identity);
                }
                PhysicalWorkTerminalEvent::DerivedReconciliationCompleted(identity) => {
                    settled.push(identity);
                }
            }
        }
        let mut residual = residual
            .into_iter()
            .map(|(identity, _stage)| identity)
            .collect::<Vec<_>>();
        sort_identities(&mut settled);
        sort_identities(&mut cancelled_before_dispatch);
        sort_identities(&mut continued_after_consumer_cancellation);
        sort_identities(&mut inspection_required);
        sort_identities(&mut released_before_dispatch);
        sort_identities(&mut residual);
        sort_identities(&mut derived_reconciliation_deferred);
        settled.dedup();
        derived_reconciliation_deferred.dedup();
        PhysicalWorkDrainObservation {
            settled: settled.into_boxed_slice(),
            cancelled_before_dispatch: cancelled_before_dispatch.into_boxed_slice(),
            continued_after_consumer_cancellation: continued_after_consumer_cancellation
                .into_boxed_slice(),
            inspection_required: inspection_required.into_boxed_slice(),
            released_before_dispatch: released_before_dispatch.into_boxed_slice(),
            residual: residual.into_boxed_slice(),
            derived_reconciliation_deferred: derived_reconciliation_deferred.into_boxed_slice(),
            evidence_capacity: self.capacity,
            evidence_overflow: self.evidence_overflow.load(Ordering::Acquire),
            safe_evidence_elided: self.safe_evidence_elided.load(Ordering::Acquire),
        }
    }

    pub(super) fn cancellation_candidates(&self) -> Vec<super::PhysicalWorkConsumerHandle> {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .filter_map(|event| match event {
                PhysicalWorkTerminalEvent::ReleasedBeforeDispatch {
                    consumer: Some(consumer),
                    ..
                } => Some(*consumer),
                _ => None,
            })
            .collect()
    }

    pub(super) fn resolve_derived_reconciliation(&self, identity: PhysicalWorkIdentity) {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(deferred) = entries.iter_mut().find(|event| {
            matches!(
                event,
                PhysicalWorkTerminalEvent::DerivedReconciliationDeferred(candidate)
                    if *candidate == identity
            )
        }) {
            *deferred = PhysicalWorkTerminalEvent::DerivedReconciliationCompleted(identity);
        }
    }
}

impl PhysicalWorkDrainObservation {
    pub const fn settled(&self) -> &[PhysicalWorkIdentity] {
        &self.settled
    }

    pub const fn cancelled_before_dispatch(&self) -> &[PhysicalWorkIdentity] {
        &self.cancelled_before_dispatch
    }

    pub const fn continued_after_consumer_cancellation(&self) -> &[PhysicalWorkIdentity] {
        &self.continued_after_consumer_cancellation
    }

    pub const fn inspection_required(&self) -> &[PhysicalWorkIdentity] {
        &self.inspection_required
    }

    pub const fn released_before_dispatch(&self) -> &[PhysicalWorkIdentity] {
        &self.released_before_dispatch
    }

    pub const fn residual(&self) -> &[PhysicalWorkIdentity] {
        &self.residual
    }

    pub const fn derived_reconciliation_deferred(&self) -> &[PhysicalWorkIdentity] {
        &self.derived_reconciliation_deferred
    }

    pub const fn evidence_overflow(&self) -> u64 {
        self.evidence_overflow
    }

    pub const fn evidence_capacity(&self) -> usize {
        self.evidence_capacity
    }

    pub const fn safe_evidence_elided(&self) -> u64 {
        self.safe_evidence_elided
    }

    pub fn exact_identity_count(&self) -> usize {
        let retained = self.settled.len()
            + self.cancelled_before_dispatch.len()
            + self.continued_after_consumer_cancellation.len()
            + self.inspection_required.len()
            + self.released_before_dispatch.len()
            + self.residual.len();
        retained.saturating_add(usize::try_from(self.safe_evidence_elided).unwrap_or(usize::MAX))
    }

    pub fn requires_inspection(&self) -> bool {
        !self.inspection_required.is_empty()
            || !self.residual.is_empty()
            || self.evidence_overflow != 0
    }
}

impl PhysicalWorkTerminalEvent {
    fn must_retain(self) -> bool {
        self.requires_inspection()
            || matches!(
                self,
                Self::ReleasedBeforeDispatch {
                    consumer: Some(_),
                    ..
                } | Self::DerivedReconciliationDeferred(_)
            )
    }

    fn requires_inspection(self) -> bool {
        match self {
            Self::Settled { fate, recovery, .. } => {
                recovery == PhysicalWorkRecoveryDisposition::InspectionRequired
                    || matches!(
                        fate,
                        PhysicalWorkEffectFate::Indeterminate
                            | PhysicalWorkEffectFate::WrittenButSchedulerRejected
                            | PhysicalWorkEffectFate::StaleOrForeignOutcome
                    )
            }
            Self::AbandonedAfterDispatch(_) => true,
            Self::CancelledBeforeDispatch(_) | Self::ReleasedBeforeDispatch { .. } => false,
            Self::DerivedReconciliationDeferred(_) | Self::DerivedReconciliationCompleted(_) => {
                false
            }
        }
    }
}

fn saturating_increment(counter: &AtomicU64) {
    let _ = counter.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
        Some(value.saturating_add(1))
    });
}

fn sort_identities(identities: &mut [PhysicalWorkIdentity]) {
    identities.sort_by_key(|identity| identity.operation().get());
}
