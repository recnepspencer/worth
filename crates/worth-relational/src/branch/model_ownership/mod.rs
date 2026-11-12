//! Declared model ownership over exact full-generation structural facts.
use crate::identity::data::{EntityId, KindId};
use std::collections::BTreeSet;

mod basis;
mod custody_observation;
pub use custody_observation::{observe_candidate_custody_reads, RelationalCandidateCustodyReads};
mod current_publications;
pub use current_publications::{
    RelationalOwnedModelOutputRoute, RelationalOwnedModelPublicationRead,
    RelationalOwnedModelPublishedRoute,
};
mod graph_admission;
mod graph_denial;
mod graph_selection;
mod invariant;
mod publication;
pub use graph_denial::{RelationalOwnedModelForeignRelation, RelationalOwnedModelSelectionDenial};
pub use graph_selection::RelationalOwnedModelGraphSelection;
pub(crate) use publication::validate_candidate_ownership;
mod selection;
mod traversal;
pub use selection::RelationalOwnedModelEntityBoundary;

/// Schema meaning only. Constructing it grants no read or mutation authority.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelationalModelOwnershipDomain {
    pub root: KindId,
    pub relations: Vec<KindId>,
    pub entities: Vec<KindId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationalModelOwnership {
    domains: Vec<RelationalModelOwnershipDomain>,
    relations: Vec<KindId>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RelationalModelOwnershipConflict {
    pub subject: EntityId,
}

impl RelationalModelOwnershipConflict {
    pub const VALUE_IDENTITY: &'static str = "worth.model-ownership.conflict.v1";
    pub fn encode(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(self)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        rmp_serde::from_slice(bytes)
    }
}

#[derive(Debug)]
pub enum RelationalModelOwnershipDenial {
    ForeignBasis,
    Custody(crate::transactions::data::TransactionCommitError),
    TargetNotCurrent,
    SchemaChanged,
    ModelRootKind { model: EntityId, actual: KindId },
    EntityUnavailable(EntityId),
    Conflict(RelationalModelOwnershipConflict),
    Read(crate::validation::data::StructuralReadError),
    Adjacency(crate::validation::data::CustomInvariantTraversalError),
    Traversal(crate::validation::data::StructuralWalkDenialKind),
    Request(worth_execution::WorkCeilingDenial),
    Native(crate::execution::RelationalExecutionDenialCause),
}

impl RelationalModelOwnership {
    pub fn new(mut domains: Vec<RelationalModelOwnershipDomain>) -> Self {
        for domain in &mut domains {
            domain.relations.sort_unstable();
            domain.relations.dedup();
            domain.entities.sort_unstable();
            domain.entities.dedup();
        }
        domains.sort_unstable();
        domains.dedup();
        let relations = domains
            .iter()
            .flat_map(|domain| domain.relations.iter().copied())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        Self { domains, relations }
    }
    pub fn combine<'a>(owners: impl IntoIterator<Item = &'a Self>) -> Self {
        Self::new(
            owners
                .into_iter()
                .flat_map(|owner| owner.domains.iter().cloned())
                .collect(),
        )
    }

    fn entity_kinds(&self) -> Vec<KindId> {
        self.domains
            .iter()
            .flat_map(|domain| domain.entities.iter().copied().chain([domain.root]))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    fn relation_kinds(&self) -> Vec<KindId> {
        self.relations.clone()
    }
    fn declares_relation(&self, kind: KindId) -> bool {
        self.relations.binary_search(&kind).is_ok()
    }
    fn relation_lookup_work(&self) -> usize {
        self.relations.len().checked_ilog2().unwrap_or(0) as usize + 1
    }
}

impl RelationalModelOwnershipDenial {
    /// Keep custody-owner interruptions visible without flattening its typed refusal.
    pub fn kernel_stop(&self) -> Option<worth_execution::MapKernelStop> {
        match self {
            Self::Request(worth_execution::WorkCeilingDenial::Stopped(stop)) => Some(*stop),
            Self::Native(cause) => cause.as_kernel_stop(),
            Self::Custody(crate::transactions::data::TransactionCommitError::Execution {
                denial,
                ..
            }) => {
                let crate::transactions::data::CommitExecutionDenialKind::Cause(cause) =
                    denial.kind;
                cause.as_kernel_stop()
            }
            _ => None,
        }
    }
}
