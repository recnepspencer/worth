//! Declared model ownership over exact full-generation structural facts.
use crate::contracts::identity::data::primitives::KindId;
use crate::contracts::identity::data::record_ids::EntityId;
use std::collections::BTreeSet;

pub use crate::contracts::branch::model_ownership::read_denial::{
    RelationalModelOwnershipConflict, RelationalModelOwnershipDenial,
};
pub use worth_relational_branch::internal_port::branch::model_ownership::custody_observation::{
    observe_candidate_custody_reads, RelationalCandidateCustodyReads,
};
use worth_relational_branch::internal_port::branch::model_ownership::native_read::ModelOwnershipReadPlan;
pub use worth_relational_branch::internal_port::branch::model_ownership::native_read::RelationalModelOwnershipDomain;
pub(crate) use worth_relational_branch::internal_port::branch::model_ownership::traversal;

pub mod graph_admission;
pub mod graph_denial;
pub mod graph_selection;
pub mod invariant;
pub mod publication;
pub use graph_denial::{RelationalOwnedModelForeignRelation, RelationalOwnedModelSelectionDenial};
pub use graph_selection::RelationalOwnedModelGraphSelection;
pub use publication::validate_candidate_ownership;

pub mod selection;
pub use selection::RelationalOwnedModelEntityBoundary;

#[derive(Clone, PartialEq, Eq)]
pub struct RelationalModelOwnership {
    plan: ModelOwnershipReadPlan,
}

impl std::fmt::Debug for RelationalModelOwnership {
    fn fmt(&self, format: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format
            .debug_struct("RelationalModelOwnership")
            .field("domains", &self.plan.domains)
            .field("relations", &self.plan.relations)
            .finish()
    }
}

impl RelationalModelOwnership {
    pub fn new(domains: Vec<RelationalModelOwnershipDomain>) -> Self {
        Self {
            plan: ModelOwnershipReadPlan::new(domains),
        }
    }
    pub fn combine<'a>(owners: impl IntoIterator<Item = &'a Self>) -> Self {
        Self {
            plan: ModelOwnershipReadPlan::combine(owners.into_iter().map(|owner| &owner.plan)),
        }
    }
}

impl AsRef<ModelOwnershipReadPlan> for RelationalModelOwnership {
    fn as_ref(&self) -> &ModelOwnershipReadPlan {
        &self.plan
    }
}
