use crate::domain_computation::execution_runtime::{
    product_world::WorthQueryProductWorldResources, WorthQueryApplicationCandidateResourceProfile,
    WorthQueryApplicationQueryResourceProfile, WorthQueryCompletedEvidenceResourceProfile,
    WorthQueryOutputDemandResourceProfile,
};
use worth_signal::facade::runtime::SignalConditionalEvaluationBudget;

use super::WorthQueryApplicationProfile;

/// Explicit finite resources and clock for one in-memory application.
#[derive(Clone)]
pub struct WorthQueryApplicationLimits {
    pub(super) world: WorthQueryProductWorldResources,
    pub(super) candidates: WorthQueryApplicationCandidateResourceProfile,
    pub(super) output_demands: WorthQueryOutputDemandResourceProfile,
    pub(super) completed_evidence: WorthQueryCompletedEvidenceResourceProfile,
    pub(super) queries: WorthQueryApplicationQueryResourceProfile,
    pub(super) conditionals: SignalConditionalEvaluationBudget,
    pub(super) profile: WorthQueryApplicationProfile,
    pub(super) maximum_publication_records: Option<std::num::NonZeroUsize>,
}

impl WorthQueryApplicationLimits {
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
            output_demands: WorthQueryOutputDemandResourceProfile::standard(),
            completed_evidence: WorthQueryCompletedEvidenceResourceProfile::standard(),
            conditionals,
            profile: WorthQueryApplicationProfile::GeneralPurpose,
            maximum_publication_records: None,
        }
    }

    pub const fn with_output_demand_resources(
        mut self,
        profile: WorthQueryOutputDemandResourceProfile,
    ) -> Self {
        self.output_demands = profile;
        self
    }

    /// Bounds completed-commit evidence, which is the declared idempotency
    /// window: once full, the oldest evidence leaves it and a replay of that
    /// commit answers that its window expired.
    pub const fn with_completed_evidence_resources(
        mut self,
        profile: WorthQueryCompletedEvidenceResourceProfile,
    ) -> Self {
        self.completed_evidence = profile;
        self
    }

    /// Selects the bounded execution policy for this application's workload.
    pub const fn with_profile(mut self, profile: WorthQueryApplicationProfile) -> Self {
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
