//! Descriptive work from the original retention owner.
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
