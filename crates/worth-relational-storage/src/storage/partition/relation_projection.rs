//! Immutable relation-kind membership facts; residency authenticates their source.
use crate::identity::data::{KindId, RelationId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionRelationProjection {
    pub relation: RelationId,
    pub kind: KindId,
    pub(in crate::internal_port::storage) outgoing: BTreeSet<usize>,
    pub incoming: BTreeSet<usize>,
}
