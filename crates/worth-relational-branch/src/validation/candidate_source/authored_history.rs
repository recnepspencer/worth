//! Authored and physical Source impact, separate from semantic maintenance effects.
pub mod catalog;
pub mod catalog_funding;
use crate::history::data::{BranchId, CommitId};
use crate::identity::data::{EntityId, KindId, RelationId, VersionId};
use crate::transactions::data::primitives::RecordRef;
use catalog::SourceHistoryCatalog;
use std::sync::Arc;
use worth_execution::{ChargedBytes, ExecutionOrderedMap};
pub mod material;
pub mod record_use;
pub use record_use::AuthoredSourceRecordUse;

/// Original authored incidence on the actual selected/working pair. This is
/// not a published semantic adjacency delta and may describe a no-op.
#[derive(Clone, Copy, Debug)]
pub struct AuthoredSourceIncidence {
    pub relation: RelationId,
    pub kind: KindId,
    pub before: Option<(EntityId, EntityId)>,
    pub after: Option<(EntityId, EntityId)>,
}
/// Ordered coordinates only; exact catalog correspondence remains separate.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ManagedSourceImpactKey {
    pub program: usize,
    pub scope: EntityId,
}
#[derive(Clone, Debug, Default)]
pub struct ManagedSourceImpactRows(pub ExecutionOrderedMap<EntityId, ()>);
impl ChargedBytes for ManagedSourceImpactKey {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
impl ChargedBytes for AuthoredSourceIncidence {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
impl ChargedBytes for ManagedSourceImpactRows {
    fn additional_charged_bytes(&self) -> u64 {
        0
    } // Cells retain their original grants.
}
#[derive(Debug)]
pub struct PreparedSourceImpactHistory {
    pub catalog: Arc<SourceHistoryCatalog>,
    pub runtime: u64,
    pub branch: BranchId,
    pub parent: Option<CommitId>,
    pub proposed: VersionId,
    #[allow(
        dead_code,
        reason = "Original diagnostic, proof and funding custody remains owned through retirement"
    )]
    pub records: ExecutionOrderedMap<RecordRef, AuthoredSourceRecordUse>,
    #[allow(
        dead_code,
        reason = "Original diagnostic, proof and funding custody remains owned through retirement"
    )]
    pub incidence: ExecutionOrderedMap<RelationId, AuthoredSourceIncidence>,
    pub impacts: ExecutionOrderedMap<ManagedSourceImpactKey, ManagedSourceImpactRows>,
    pub _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
