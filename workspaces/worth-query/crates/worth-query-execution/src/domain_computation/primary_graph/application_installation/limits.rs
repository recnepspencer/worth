use crate::domain_computation::execution_runtime::{
    product_world::WorthQueryProductWorldResources, WorthQueryApplicationCandidateResourceProfile,
    WorthQueryApplicationQueryResourceProfile,
};
use worth_signal::facade::runtime::SignalConditionalEvaluationBudget;

/// Explicit finite resources and clock for one in-memory application.
#[derive(Clone)]
pub struct WorthQueryInMemoryApplicationLimits {
    pub(super) world: WorthQueryProductWorldResources,
    pub(super) candidates: WorthQueryApplicationCandidateResourceProfile,
    pub(super) queries: WorthQueryApplicationQueryResourceProfile,
    pub(super) conditionals: SignalConditionalEvaluationBudget,
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
        }
    }
}
