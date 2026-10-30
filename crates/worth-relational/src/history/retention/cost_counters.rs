use super::RelationalRetentionObligationKind;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RelationalRetentionCostCounters {
    pub observation_acquires: u64,
    pub observation_releases: u64,
    pub transaction_acquires: u64,
    pub transaction_releases: u64,
    pub candidate_acquires: u64,
    pub candidate_releases: u64,
    pub performed_settlement_acquires: u64,
    pub performed_settlement_releases: u64,
    pub external_pin_acquires: u64,
    pub external_pin_releases: u64,
    pub head_installs: u64,
    pub head_transfers: u64,
    pub retired_root_enqueues: u64,
    pub reclamation_roots_examined: u64,
    /// Retired-root rows visited to decide derived-generation reader custody.
    pub index_retention_roots_examined: u64,
    pub reclamation_roots_reclaimed: u64,
    pub reclamation_unique_authoritative_bytes: u64,
}

impl RelationalRetentionCostCounters {
    pub(crate) fn saturating_add(self, other: Self) -> Self {
        macro_rules! add_fields {
            ($($field:ident),+ $(,)?) => {
                Self { $($field: self.$field.saturating_add(other.$field)),+ }
            };
        }
        add_fields!(
            observation_acquires,
            observation_releases,
            transaction_acquires,
            transaction_releases,
            candidate_acquires,
            candidate_releases,
            performed_settlement_acquires,
            performed_settlement_releases,
            external_pin_acquires,
            external_pin_releases,
            head_installs,
            head_transfers,
            retired_root_enqueues,
            reclamation_roots_examined,
            index_retention_roots_examined,
            reclamation_roots_reclaimed,
            reclamation_unique_authoritative_bytes,
        )
    }

    pub(crate) fn saturating_delta_since(self, baseline: Self) -> Self {
        macro_rules! delta_fields {
            ($($field:ident),+ $(,)?) => {
                Self { $($field: self.$field.saturating_sub(baseline.$field)),+ }
            };
        }
        delta_fields!(
            observation_acquires,
            observation_releases,
            transaction_acquires,
            transaction_releases,
            candidate_acquires,
            candidate_releases,
            performed_settlement_acquires,
            performed_settlement_releases,
            external_pin_acquires,
            external_pin_releases,
            head_installs,
            head_transfers,
            retired_root_enqueues,
            reclamation_roots_examined,
            index_retention_roots_examined,
            reclamation_roots_reclaimed,
            reclamation_unique_authoritative_bytes,
        )
    }

    pub(crate) fn maintenance_only(self) -> Self {
        Self {
            reclamation_roots_examined: self.reclamation_roots_examined,
            index_retention_roots_examined: self.index_retention_roots_examined,
            reclamation_roots_reclaimed: self.reclamation_roots_reclaimed,
            reclamation_unique_authoritative_bytes: self.reclamation_unique_authoritative_bytes,
            ..Self::default()
        }
    }
}

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

pub(super) fn record_reclamation(
    counters: &mut RelationalRetentionCostCounters,
    examined: u64,
    reclaimed: u64,
    unique_bytes: u64,
) {
    counters.reclamation_roots_examined =
        counters.reclamation_roots_examined.saturating_add(examined);
    counters.reclamation_roots_reclaimed = counters
        .reclamation_roots_reclaimed
        .saturating_add(reclaimed);
    counters.reclamation_unique_authoritative_bytes = counters
        .reclamation_unique_authoritative_bytes
        .saturating_add(unique_bytes);
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
