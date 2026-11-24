//! Native payload owners retain their own original-request funding.
mod extra;
mod funding;
mod metadata;
#[cfg(test)]
mod tests;
use crate::identity::data::EntityId;
pub(crate) use extra::{current_extra_birth_watermark, EntityExtra, RelationExtra};
pub(crate) use funding::ExtraPayloadDenial;
pub(crate) use metadata::{VersionedEntityMetadata, VersionedRelationMetadata};
#[cfg(test)]
pub(crate) use tests::assert_unique_extra_move_funding;
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
