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

#[cfg(test)]
mod commit_tests;
mod field_pinning;
mod frontier_reads;
mod pinning;
mod record_reads;
#[cfg(test)]
mod tests;

/// An exact root, its schema/frontiers/revisions, frozen symbols, and selected
/// index handles. Cloning establishes another immutable observer; it copies
/// neither rows nor an index. No runtime, request, or registry guard is retained.
/// Snapshot-backed pins carry their handle. Observation-backed pins carry `()`;
/// only snapshot-backed pins expose snapshot/plan doors, by type.
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
/// Observation-backed pins have no snapshot query context.
/// ```compile_fail,E0599
/// use worth_relational::facade::runtime::RelationalPinnedRead;
/// fn context(read: &RelationalPinnedRead<()>) { read.query_context(); }
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
pub struct RelationalPinnedRead<Snapshot = SnapshotHandle> {
    pub(super) current_version: crate::identity::data::VersionId,
    pub(super) instrumentation: crate::runtime::RuntimeInstrumentation,
    pub(super) snapshot: Snapshot,
    pub(super) basis: VisibilitySnapshotBasis,
    pub(super) context: QueryPlanContextId,
    pub(super) symbols: StringInterner,
    pub(super) indexes: BTreeMap<DerivedIndexId, PinnedIndex>,
}

#[derive(Clone, Debug)]
pub(crate) struct PinnedIndex {
    pub definition: Arc<DerivedIndexDefinition>,
    pub generation: Option<Arc<DerivedIndexGeneration>>,
}

#[derive(Debug)]
pub enum RelationalPinnedReadAdmissionDenial {
    Request(worth_execution::MapKernelStop),
    SnapshotUnavailable,
    IndexNotInstalled(DerivedIndexId),
}

impl RelationalPinnedRead {
    /// Schema and canonical source revisions captured at pin time.
    pub fn query_context(&self) -> &QueryPlanContextId {
        &self.context
    }

    /// Metadata of the exact owner-issued snapshot, including its source identity.
    pub fn snapshot(&self) -> &SnapshotHandle {
        &self.snapshot
    }
}

impl<Snapshot> RelationalPinnedRead<Snapshot> {
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

impl<Snapshot> RelationalPinnedRead<Snapshot> {
    pub(crate) fn runtime_instance_id(&self) -> u64 {
        self.context.runtime_instance_id
    }
    pub(crate) fn pinned_basis(&self) -> &VisibilitySnapshotBasis {
        &self.basis
    }
    pub(crate) fn pinned_current_version(&self) -> crate::identity::data::VersionId {
        self.current_version
    }
    pub(crate) fn pinned_instrumentation(&self) -> &crate::runtime::RuntimeInstrumentation {
        &self.instrumentation
    }
    pub(crate) fn selected_index(&self, index: DerivedIndexId) -> Option<&PinnedIndex> {
        self.indexes.get(&index)
    }
}

impl<Snapshot> RelationalPinnedRead<Snapshot> {
    pub(crate) fn performance_access(&self) -> crate::performance::PerformanceAccess<'_> {
        crate::performance::PerformanceAccess::from_instrumentation(&self.instrumentation)
    }
}
impl<Snapshot> super::super::projection::ProjectionRecordSource for RelationalPinnedRead<Snapshot> {
    fn selected_root(&self) -> Option<&Arc<crate::branch::RelationalBranchRoot>> {
        Some(self.basis.root())
    }
    fn is_exact_basis(&self) -> bool {
        true
    }
    fn version_id(&self) -> crate::identity::data::VersionId {
        self.basis.version_id()
    }
}
