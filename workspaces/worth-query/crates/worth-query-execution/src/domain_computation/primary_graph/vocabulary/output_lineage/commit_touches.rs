//! Moved whole from `domain_computation::primary_graph::output_lineage::invalidation::commit_touches`: the items below and every impl they own.
use std::collections::BTreeSet;
use worth_relational::facade::identity::EntityId;

/// The records a reader watches. Conditional source records are entities;
/// `every_commit` covers whole-graph dependencies and bootstrap catch-up.
#[derive(Clone, Copy)]
pub struct CommitTouchInterest<'a> {
    pub(in crate::domain_computation::primary_graph) entities: CommitTouchEntities<'a>,
    pub(in crate::domain_computation::primary_graph) every_commit: bool,
}

/// Borrowed interest uses the same canonical touch stream for sets and bounded read dependencies.
#[derive(Clone, Copy)]
pub enum CommitTouchEntities<'a> {
    Set(&'a BTreeSet<EntityId>),
    Slice(&'a [EntityId]),
}

impl<'a> From<&'a BTreeSet<EntityId>> for CommitTouchEntities<'a> {
    fn from(value: &'a BTreeSet<EntityId>) -> Self {
        Self::Set(value)
    }
}

impl<'a> From<&'a [EntityId]> for CommitTouchEntities<'a> {
    fn from(value: &'a [EntityId]) -> Self {
        Self::Slice(value)
    }
}

impl CommitTouchEntities<'_> {
    pub(in crate::domain_computation::primary_graph) fn contains(&self, id: &EntityId) -> bool {
        match self {
            Self::Set(v) => v.contains(id),
            Self::Slice(v) => v.contains(id),
        }
    }
}

impl<'a> CommitTouchInterest<'a> {
    #[doc(hidden)]
    pub fn new_for_test(entities: CommitTouchEntities<'a>, every_commit: bool) -> Self {
        Self {
            entities,
            every_commit,
        }
    }
}
