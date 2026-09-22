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
    pub(super) maximum_publication_records: Option<std::num::NonZeroUsize>,
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
            maximum_publication_records: None,
        }
    }

    /// Selects the bounded execution policy for this application's workload.
    pub const fn with_profile(mut self, profile: WorthQueryInMemoryApplicationProfile) -> Self {
        self.profile = profile;
        self
    }

    /// Overrides only the finite per-commit record ceiling. Candidate admission,
    /// invariant validation, and all other selected profile policies are unchanged.
    pub const fn with_maximum_publication_records(
        mut self,
        maximum: std::num::NonZeroUsize,
    ) -> Self {
        self.maximum_publication_records = Some(maximum);
        self
    }
}
