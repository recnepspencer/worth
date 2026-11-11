//! Native payload owners retain their own original-request funding.
mod extra;
mod funding;
mod metadata;
#[cfg(test)]
mod tests;
use crate::identity::data::EntityId;
pub(crate) use extra::{EntityExtra, RelationExtra};
pub(crate) use funding::ExtraPayloadDenial;
pub(crate) use metadata::{VersionedEntityMetadata, VersionedRelationMetadata};
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct LifecycleCounts {
    pub(crate) live: usize,
    pub(crate) unavailable: usize,
    pub(crate) deleted: usize,
    pub(crate) reusable: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct RelationEndpoints {
    pub(crate) source: EntityId,
    pub(crate) target: EntityId,
}
