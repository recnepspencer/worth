use super::RelationalRetentionObligationKind;
use std::sync::atomic::{AtomicU64, Ordering};

pub use snapshot::RelationalRetentionCostCounters;
pub(crate) use worth_relational_contracts::internal_port::history::retention::cost_counters::snapshot;

#[derive(Debug, Default)]
pub(super) struct RelationalRetentionAtomicCounters {
    observation_acquires: AtomicU64,
    observation_releases: AtomicU64,
    transaction_acquires: AtomicU64,
    transaction_releases: AtomicU64,
    candidate_acquires: AtomicU64,
    candidate_releases: AtomicU64,
    performed_settlement_acquires: AtomicU64,
    performed_settlement_releases: AtomicU64,
    external_pin_acquires: AtomicU64,
    external_pin_releases: AtomicU64,
}

impl RelationalRetentionAtomicCounters {
    pub(super) fn record_acquire(&self, kind: RelationalRetentionObligationKind) {
        self.counter(kind, true).fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_release(&self, kind: RelationalRetentionObligationKind) {
        self.counter(kind, false).fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn snapshot(&self) -> RelationalRetentionCostCounters {
        RelationalRetentionCostCounters {
            observation_acquires: self.observation_acquires.load(Ordering::Relaxed),
            observation_releases: self.observation_releases.load(Ordering::Relaxed),
            transaction_acquires: self.transaction_acquires.load(Ordering::Relaxed),
            transaction_releases: self.transaction_releases.load(Ordering::Relaxed),
            candidate_acquires: self.candidate_acquires.load(Ordering::Relaxed),
            candidate_releases: self.candidate_releases.load(Ordering::Relaxed),
            performed_settlement_acquires: self
                .performed_settlement_acquires
                .load(Ordering::Relaxed),
            performed_settlement_releases: self
                .performed_settlement_releases
                .load(Ordering::Relaxed),
            external_pin_acquires: self.external_pin_acquires.load(Ordering::Relaxed),
            external_pin_releases: self.external_pin_releases.load(Ordering::Relaxed),
            ..RelationalRetentionCostCounters::default()
        }
    }

    fn counter(&self, kind: RelationalRetentionObligationKind, acquire: bool) -> &AtomicU64 {
        match (kind, acquire) {
            (RelationalRetentionObligationKind::Observation, true) => &self.observation_acquires,
            (RelationalRetentionObligationKind::Observation, false) => &self.observation_releases,
            (RelationalRetentionObligationKind::Transaction, true) => &self.transaction_acquires,
            (RelationalRetentionObligationKind::Transaction, false) => &self.transaction_releases,
            (RelationalRetentionObligationKind::Candidate, true) => &self.candidate_acquires,
            (RelationalRetentionObligationKind::Candidate, false) => &self.candidate_releases,
            (RelationalRetentionObligationKind::PerformedSettlement, true) => {
                &self.performed_settlement_acquires
            }
            (RelationalRetentionObligationKind::PerformedSettlement, false) => {
                &self.performed_settlement_releases
            }
            (RelationalRetentionObligationKind::ExternalComponentBasis, true) => {
                &self.external_pin_acquires
            }
            (RelationalRetentionObligationKind::ExternalComponentBasis, false) => {
                &self.external_pin_releases
            }
        }
    }
}

pub(super) fn try_record_reclamation(
    counters: &mut RelationalRetentionCostCounters,
    examined: u64,
    reclaimed: u64,
    unique_bytes: u64,
) -> Result<(), crate::storage::substrate::StorageAllocationOverflow> {
    use crate::storage::substrate::StorageAllocationOverflow;
    let next_examined = counters
        .reclamation_roots_examined
        .checked_add(examined)
        .ok_or(StorageAllocationOverflow)?;
    let next_reclaimed = counters
        .reclamation_roots_reclaimed
        .checked_add(reclaimed)
        .ok_or(StorageAllocationOverflow)?;
    let next_unique_bytes = counters
        .reclamation_unique_authoritative_bytes
        .checked_add(unique_bytes)
        .ok_or(StorageAllocationOverflow)?;
    counters.reclamation_roots_examined = next_examined;
    counters.reclamation_roots_reclaimed = next_reclaimed;
    counters.reclamation_unique_authoritative_bytes = next_unique_bytes;
    Ok(())
}

pub(super) fn record_acquire(
    counters: &mut RelationalRetentionCostCounters,
    kind: RelationalRetentionObligationKind,
) {
    match kind {
        RelationalRetentionObligationKind::Observation => {
            counters.observation_acquires = counters.observation_acquires.saturating_add(1)
        }
        RelationalRetentionObligationKind::Transaction => {
            counters.transaction_acquires = counters.transaction_acquires.saturating_add(1)
        }
        RelationalRetentionObligationKind::Candidate => {
            counters.candidate_acquires = counters.candidate_acquires.saturating_add(1)
        }
        RelationalRetentionObligationKind::PerformedSettlement => {
            counters.performed_settlement_acquires =
                counters.performed_settlement_acquires.saturating_add(1)
        }
        RelationalRetentionObligationKind::ExternalComponentBasis => {
            counters.external_pin_acquires = counters.external_pin_acquires.saturating_add(1)
        }
    }
}

pub(super) fn record_release(
    counters: &mut RelationalRetentionCostCounters,
    kind: RelationalRetentionObligationKind,
) {
    match kind {
        RelationalRetentionObligationKind::Observation => {
            counters.observation_releases = counters.observation_releases.saturating_add(1)
        }
        RelationalRetentionObligationKind::Transaction => {
            counters.transaction_releases = counters.transaction_releases.saturating_add(1)
        }
        RelationalRetentionObligationKind::Candidate => {
            counters.candidate_releases = counters.candidate_releases.saturating_add(1)
        }
        RelationalRetentionObligationKind::PerformedSettlement => {
            counters.performed_settlement_releases =
                counters.performed_settlement_releases.saturating_add(1)
        }
        RelationalRetentionObligationKind::ExternalComponentBasis => {
            counters.external_pin_releases = counters.external_pin_releases.saturating_add(1)
        }
    }
}
