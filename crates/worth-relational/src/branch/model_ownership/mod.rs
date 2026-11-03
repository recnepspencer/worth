//! Declared model ownership over exact full-generation structural facts.
use crate::identity::data::{EntityId, KindId};
use std::collections::BTreeSet;

mod basis;
mod graph_denial;
mod graph_selection;
mod invariant;
pub use graph_denial::{RelationalOwnedModelForeignRelation, RelationalOwnedModelSelectionDenial};
pub use graph_selection::RelationalOwnedModelGraphSelection;
mod selection;
mod traversal;
pub use selection::RelationalOwnedModelEntityBoundary;

/// Schema meaning only. Constructing it grants no read or mutation authority.
#[derive(Clone, Debug)]
pub struct RelationalModelOwnershipDomain {
    pub root: KindId,
    pub relations: Vec<KindId>,
    pub entities: Vec<KindId>,
}

#[derive(Clone, Debug)]
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
    pub fn new(domains: Vec<RelationalModelOwnershipDomain>) -> Self {
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
