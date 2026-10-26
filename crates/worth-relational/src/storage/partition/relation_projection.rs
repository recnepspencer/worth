//! Immutable relation-kind membership facts; residency authenticates their source.
use crate::identity::data::{KindId, RelationId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RegionRelationProjection {
    pub(in crate::storage) relation: RelationId,
    pub(in crate::storage) kind: KindId,
    pub(in crate::storage) outgoing: BTreeSet<usize>,
    pub(in crate::storage) incoming: BTreeSet<usize>,
}
