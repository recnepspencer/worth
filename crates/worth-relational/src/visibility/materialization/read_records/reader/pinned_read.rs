//! Immutable native custody independent of the installed runtime's borrow.
use crate::indexes::data::{
    DerivedIndexDefinition, DerivedIndexGeneration, DerivedIndexGenerationId, DerivedIndexId,
};
use crate::query::data::QueryPlanContextId;
use crate::schema::data::RelationalSchemaRegistry;
use crate::snapshots::data::SnapshotHandle;
use crate::symbols::data::StringInterner;
use crate::visibility::snapshot_states::VisibilitySnapshotBasis;
use std::{collections::BTreeMap, sync::Arc};

mod pinning;
#[cfg(test)]
mod tests;

/// An exact root, its schema/frontiers/revisions, frozen symbols, and selected
/// index handles. Cloning establishes another immutable observer; it copies
/// neither rows nor an index. No runtime, request, or registry guard is retained.
///
/// Only the native owner can construct this capability.
/// ```compile_fail,E0451
/// use worth_relational::facade::runtime::RelationalPinnedRead;
/// fn forged() -> RelationalPinnedRead {
///     RelationalPinnedRead { snapshot: panic!(), ..panic!() }
/// }
/// ```
/// ```
/// use worth_relational::facade::{runtime::RelationalRuntime, snapshots::SnapshotHandle};
/// use worth_execution::ExecutionRequest;
/// fn owned(runtime: &RelationalRuntime, snapshot: &SnapshotHandle, request: &ExecutionRequest<'_, '_>) {
///     let _ = runtime.read_truth().pin_read_capability(snapshot, &[], request);
/// }
/// ```
/// No runtime borrow can be extracted from a pinned read.
/// ```compile_fail,E0599
/// use worth_relational::facade::runtime::{RelationalPinnedRead, RelationalRuntime};
/// fn runtime(read: &RelationalPinnedRead) -> &RelationalRuntime { read.runtime() }
/// ```
/// ```
/// use worth_relational::facade::runtime::RelationalPinnedRead;
/// use worth_relational::facade::snapshots::SnapshotHandle;
/// fn snapshot(read: &RelationalPinnedRead) -> &SnapshotHandle { read.snapshot() }
/// ```
#[derive(Clone, Debug)]
pub struct RelationalPinnedRead {
    pub(super) snapshot: SnapshotHandle,
    pub(super) basis: VisibilitySnapshotBasis,
    pub(super) context: QueryPlanContextId,
    pub(super) symbols: StringInterner,
    pub(super) indexes: BTreeMap<DerivedIndexId, PinnedIndex>,
}

#[derive(Clone, Debug)]
pub(super) struct PinnedIndex {
    pub definition: Arc<DerivedIndexDefinition>,
    pub generation: Option<Arc<DerivedIndexGeneration>>,
}

#[derive(Debug)]
pub enum RelationalPinnedReadAdmissionDenial {
    Request(super::QueryReadExecutionStop),
    SnapshotUnavailable,
    IndexNotInstalled(DerivedIndexId),
}

impl RelationalPinnedRead {
    /// Metadata of the exact owner-issued snapshot, including its source identity.
    pub fn snapshot(&self) -> &SnapshotHandle {
        &self.snapshot
    }
    /// Schema and canonical source revisions captured at pin time.
    pub fn query_context(&self) -> &QueryPlanContextId {
        &self.context
    }
    /// Immutable schema carried by the selected root, never current configuration.
    pub fn schema(&self) -> &RelationalSchemaRegistry {
        self.basis.root().schema_authority().registry()
    }
    /// Frozen persistent symbol maps; a later intern or replacement cannot change them.
    pub fn symbols(&self) -> &StringInterner {
        &self.symbols
    }
    /// The semantic definition captured with this selected index.
    pub fn index_definition(&self, index: DerivedIndexId) -> Option<&DerivedIndexDefinition> {
        self.indexes
            .get(&index)
            .map(|entry| entry.definition.as_ref())
    }
    /// `None` retains a cold index's absence at pin time.
    pub fn index_generation(&self, index: DerivedIndexId) -> Option<DerivedIndexGenerationId> {
        self.indexes
            .get(&index)?
            .generation
            .as_ref()
            .map(|generation| generation.generation_id)
    }
}
