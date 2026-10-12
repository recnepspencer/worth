//! Resources and observations used by observer-backed checkpoint proofs.
pub(in crate::checkpoint_recovery) use super::execution_policy::CHECKPOINT_EXECUTION_POLICY;
use super::*;

pub(in crate::checkpoint_recovery) fn limits_with_history_room(
    retained_composite_commits: u64,
    active_observations: u64,
    unique_exact_component_pins: u64,
    invalidation: worth_query_host::facade::runtime::WorthQueryInvalidationResources,
    candidates: WorthQueryApplicationCandidateResourceProfile,
    history_metadata_bytes: u64,
) -> WorthQueryInMemoryApplicationLimits {
    super::limits_with_history_and_policy(
        retained_composite_commits,
        active_observations,
        unique_exact_component_pins,
        invalidation,
        candidates,
        history_metadata_bytes,
        super::execution_policy::CHECKPOINT_EXECUTION_POLICY,
    )
}

impl crate::producer::InitialPlanarProvider {
    pub(crate) fn with_uniform_decimal_key_width(mut self) -> Self {
        // Twenty decimal digits for every source identity, independent of hashing.
        self.key_mask = 3 << 62;
        self
    }
}
