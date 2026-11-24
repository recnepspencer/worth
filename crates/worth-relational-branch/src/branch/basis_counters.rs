/// Exact ordinary-work counters for Phase-6 basis admission and retention.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RelationalBranchBasisCostCounters {
    pub basis_observations: u64,
    pub descriptor_resolution_attempts: u64,
    pub readmission_successes: u64,
    pub readmission_denials: u64,
    pub stale_readmission_denials: u64,
    pub external_retention_acquires: u64,
    pub external_retention_releases: u64,
    pub external_retention_drop_releases: u64,
    pub retained_basis_registry_entries: u64,
    pub retained_basis_registry_key_lookups: u64,
    pub retained_basis_registry_mutations: u64,
}

impl RelationalBranchBasisCostCounters {
    pub(crate) fn record_readmission(
        &mut self,
        result: &Result<
            crate::branch::basis::AdmittedRelationalBranchBasis,
            crate::branch::basis_denial::RelationalBranchBasisDenial,
        >,
    ) {
        self.descriptor_resolution_attempts = self.descriptor_resolution_attempts.saturating_add(1);
        match result {
            Ok(_) => self.readmission_successes = self.readmission_successes.saturating_add(1),
            Err(denial) => {
                self.readmission_denials = self.readmission_denials.saturating_add(1);
                if matches!(
                    denial,
                    crate::branch::basis_denial::RelationalBranchBasisDenial::StaleReferenceGeneration
                ) {
                    self.stale_readmission_denials =
                        self.stale_readmission_denials.saturating_add(1);
                }
            }
        }
    }
}
