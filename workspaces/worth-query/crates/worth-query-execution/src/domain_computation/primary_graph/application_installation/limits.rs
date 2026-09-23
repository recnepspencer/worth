use crate::domain_computation::execution_runtime::{
    product_world::WorthQueryProductWorldResources, WorthQueryApplicationCandidateResourceProfile,
    WorthQueryApplicationQueryResourceProfile,
};
use worth_signal::facade::runtime::SignalConditionalEvaluationBudget;

use super::WorthQueryInMemoryApplicationProfile;

/// Explicit finite resources and clock for one in-memory application.
#[derive(Clone)]
pub struct WorthQueryInMemoryApplicationLimits {
    pub(super) world: WorthQueryProductWorldResources,
    pub(super) candidates: WorthQueryApplicationCandidateResourceProfile,
    pub(super) queries: WorthQueryApplicationQueryResourceProfile,
    pub(super) conditionals: SignalConditionalEvaluationBudget,
    pub(super) profile: WorthQueryInMemoryApplicationProfile,
    pub(super) relational_transaction_capacity: Option<RelationalTransactionCapacity>,
}

#[derive(Clone, Copy)]
pub(super) struct RelationalTransactionCapacity {
    pub(super) patch_records: usize,
    pub(super) footprint_loci: usize,
    pub(super) touched_entities: usize,
}

impl WorthQueryInMemoryApplicationLimits {
    pub const fn new(
        world: WorthQueryProductWorldResources,
        candidates: WorthQueryApplicationCandidateResourceProfile,
        queries: WorthQueryApplicationQueryResourceProfile,
        conditionals: SignalConditionalEvaluationBudget,
    ) -> Self {
        Self {
            world,
            candidates,
            queries,
            conditionals,
            profile: WorthQueryInMemoryApplicationProfile::GeneralPurpose,
            relational_transaction_capacity: None,
        }
    }

    /// Selects the bounded execution policy for this application's workload.
    pub const fn with_profile(mut self, profile: WorthQueryInMemoryApplicationProfile) -> Self {
        self.profile = profile;
        self
    }

    /// Explicitly admits a larger, finite native setup transaction when the
    /// selected profile's default publication limits are insufficient.
    pub const fn with_relational_transaction_capacity(
        mut self,
        patch_records: usize,
        footprint_loci: usize,
        touched_entities: usize,
    ) -> Self {
        self.relational_transaction_capacity = Some(RelationalTransactionCapacity {
            patch_records,
            footprint_loci,
            touched_entities,
        });
        self
    }
}
