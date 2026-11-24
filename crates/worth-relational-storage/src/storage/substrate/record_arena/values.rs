//! Native payload owners retain their own original-request funding.
pub mod extra;
pub mod funding;
pub mod metadata;
#[cfg(test)]
mod tests;
use crate::identity::data::EntityId;
pub use extra::{current_extra_birth_watermark, EntityExtra, RelationExtra};
pub use funding::ExtraPayloadDenial;
pub use metadata::{VersionedEntityMetadata, VersionedRelationMetadata};
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LifecycleCounts {
    pub live: usize,
    pub unavailable: usize,
    pub deleted: usize,
    pub reusable: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RelationEndpoints {
    pub source: EntityId,
    pub target: EntityId,
}
