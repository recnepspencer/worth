//! Authored and physical Source impact, separate from semantic maintenance effects.
pub(crate) mod catalog;
use crate::history::data::{BranchId, CommitId};
use crate::identity::data::{EntityId, KindId, RelationId, VersionId};
use crate::transactions::data::RecordRef;
use catalog::SourceHistoryCatalog;
use std::sync::Arc;
use worth_execution::{ChargedBytes, ExecutionOrderedMap};
mod admission;
mod collection;
mod record_use;
pub(crate) use record_use::AuthoredSourceRecordUse;

/// Original authored incidence on the actual selected/working pair. This is
/// not a published semantic adjacency delta and may describe a no-op.
#[derive(Clone, Copy, Debug)]
pub(crate) struct AuthoredSourceIncidence {
    pub(crate) relation: RelationId,
    pub(crate) kind: KindId,
    pub(crate) before: Option<(EntityId, EntityId)>,
    pub(crate) after: Option<(EntityId, EntityId)>,
}
/// Ordered coordinates only; exact catalog correspondence remains separate.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ManagedSourceImpactKey {
    pub(crate) program: usize,
    pub(crate) scope: EntityId,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct ManagedSourceImpactRows(pub(crate) ExecutionOrderedMap<EntityId, ()>);
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
pub(crate) struct PreparedSourceImpactHistory {
    pub(crate) catalog: Arc<SourceHistoryCatalog>,
    pub(crate) runtime: u64,
    pub(crate) branch: BranchId,
    pub(crate) parent: Option<CommitId>,
    pub(crate) proposed: VersionId,
    pub(crate) records: ExecutionOrderedMap<RecordRef, AuthoredSourceRecordUse>,
    pub(crate) incidence: ExecutionOrderedMap<RelationId, AuthoredSourceIncidence>,
    pub(crate) impacts: ExecutionOrderedMap<ManagedSourceImpactKey, ManagedSourceImpactRows>,
    pub(crate) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

mod preparation;
pub(super) use preparation::prepare;
