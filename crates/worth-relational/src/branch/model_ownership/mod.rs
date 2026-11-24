//! Declared model ownership over exact full-generation structural facts.
use crate::identity::data::{EntityId, KindId};
use std::collections::BTreeSet;

mod basis;
pub(crate) mod native_read;
use native_read::ModelOwnershipReadPlan;
pub use native_read::RelationalModelOwnershipDomain;
pub(crate) mod read_denial;
pub use read_denial::{RelationalModelOwnershipConflict, RelationalModelOwnershipDenial};
mod custody_observation;
pub use custody_observation::{observe_candidate_custody_reads, RelationalCandidateCustodyReads};
pub(crate) mod current_publications;
mod graph_admission;
mod graph_denial;
mod graph_selection;
mod invariant;
mod publication;
pub use graph_denial::{RelationalOwnedModelForeignRelation, RelationalOwnedModelSelectionDenial};
pub use graph_selection::RelationalOwnedModelGraphSelection;
pub(crate) use publication::validate_candidate_ownership;
mod runtime_publications;
mod runtime_selection;
mod selection;
mod traversal;
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
