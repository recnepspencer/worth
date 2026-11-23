//! Immutable native custody independent of the installed runtime's borrow.
use crate::indexes::access::selected_index::PinnedIndex;
use crate::indexes::data::{DerivedIndexDefinition, DerivedIndexGenerationId, DerivedIndexId};
use crate::query::data::QueryPlanContextId;
use crate::schema::data::RelationalSchemaRegistry;
use crate::snapshots::data::SnapshotHandle;
use crate::symbols::data::StringInterner;
use crate::visibility::snapshot_states::{
    ResidentVisibilitySnapshotBasis, VisibilitySnapshotBasis,
};
use std::{collections::BTreeMap, sync::Arc};

mod aggregate_reads;
mod candidate;
mod derived_source;
mod frontier_reads;
mod index_lookups;
mod pinning;
use crate::indexes::data::applied_candidate_qualification::CandidateReadPhase;
pub use crate::indexes::data::applied_candidate_qualification::RelationalAppliedCandidateReadQualification;
pub use candidate::RelationalAppliedCandidateReadIdentity;
mod record_reads;
#[cfg(test)]
mod tests;

/// An exact root, its schema/frontiers/revisions, isolated dictionary namespace,
/// and selected index handles. Reads demand only reached regions and symbol
/// pages. Cloning establishes another immutable observer; it copies
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
    pub(super) applied_changes: Option<Arc<candidate::AppliedReadChanges>>,
    pub(super) current_version: crate::identity::data::VersionId,
    pub(super) instrumentation:
        crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
    pub(super) snapshot: Snapshot,
    pub(super) basis: ResidentVisibilitySnapshotBasis,
    pub(super) context: QueryPlanContextId,
    pub(super) indexes: BTreeMap<DerivedIndexId, PinnedIndex>,
}

pub use super::pin_denial::RelationalPinnedReadAdmissionDenial;

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
    /// Restrict an exact read to physical projection custody. The returned pin
    /// retains its actual root and evidence basis, and exposes no operational
    /// snapshot handle or snapshot query-plan execution door.
    pub fn into_projection_read(self) -> RelationalPinnedRead<()> {
        let Self {
            applied_changes,
            current_version,
            instrumentation,
            snapshot: _,
            basis,
            context,
            indexes,
        } = self;
        RelationalPinnedRead {
            applied_changes,
            current_version,
            instrumentation,
            snapshot: (),
            basis,
            context,
            indexes,
        }
    }
    /// Descriptive context of the retained root. This does not issue a snapshot
    /// handle; applied roots retain their explicit applied evidence basis.
    pub fn read_context(&self) -> &QueryPlanContextId {
        &self.context
    }

    /// Immutable schema carried by the selected root, never current configuration.
    pub fn schema(&self) -> &RelationalSchemaRegistry {
        self.basis.root().schema_authority().registry()
    }
    /// Persistent snapshot of this pin's resident dictionary meanings. Record
    /// reads admit their region's symbol closure; later outside interns or
    /// dictionary replacement cannot alter this pin's namespace. Retain the
    /// returned value when borrowing text across statements.
    pub fn symbols(&self) -> StringInterner {
        self.basis.symbols().interner_snapshot()
    }

    /// Resolve one symbol against this pin's exact dictionary, admitting only
    /// its cold page under the original read request. The callback borrows
    /// native text custody without cloning the text or a whole dictionary.
    pub fn with_symbol_text<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbol: crate::symbols::data::Symbol,
        inspect: impl FnOnce(Option<&crate::symbols::data::StoredSymbolText>) -> T,
    ) -> Result<T, crate::execution::RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)?;
        self.basis
            .symbols()
            .demand_symbol_ids(execution, [symbol])?;
        let symbols = self.symbols();
        Ok(inspect(symbols.retained_text(symbol)))
    }
    /// The semantic definition captured with this selected index.
    pub fn index_definition(&self, index: DerivedIndexId) -> Option<&DerivedIndexDefinition> {
        self.indexes
            .get(&index)
            .map(|entry| entry.definition.as_ref())
    }
    /// `None` records genuine generation absence after checkpoint admission.
    pub fn index_generation(&self, index: DerivedIndexId) -> Option<DerivedIndexGenerationId> {
        self.indexes
            .get(&index)?
            .generation
            .as_ref()
            .map(|generation| generation.generation_id)
    }
}

impl<Snapshot> RelationalPinnedRead<Snapshot> {
    pub(crate) fn applied_field_image(
        &self,
    ) -> Option<&crate::indexes::authority::maintenance::AppliedFieldImage> {
        self.applied_changes
            .as_ref()
            .map(|changes| changes.fields.as_ref())
    }
    pub(crate) fn candidate_aggregate(
        &self,
    ) -> Option<&crate::aggregate_identity::AggregateCandidateRead> {
        self.applied_changes
            .as_ref()
            .and_then(|changes| changes.aggregate.as_ref())
    }
    pub(crate) fn runtime_instance_id(&self) -> u64 {
        self.context.runtime_instance_id
    }
    pub(crate) fn pinned_basis(&self) -> &VisibilitySnapshotBasis {
        self.basis.basis()
    }
    pub(crate) fn resident_basis(&self) -> &ResidentVisibilitySnapshotBasis {
        &self.basis
    }
    pub(crate) fn pinned_current_version(&self) -> crate::identity::data::VersionId {
        self.current_version
    }
    pub(crate) fn pinned_instrumentation(
        &self,
    ) -> &crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation {
        &self.instrumentation
    }
}

impl<Snapshot> super::super::projection::ProjectionRecordSource for RelationalPinnedRead<Snapshot> {
    // A pin retains the exact root and its dictionary. Each operation admits
    // only the regions it reaches before borrowing their materialized rows.
    fn demand_all_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        self.basis
            .demand_regions_full(execution, self.basis.root().partition_ids_iter())
            .map_err(Into::into)
    }
    fn demand_regions_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partitions: impl IntoIterator<Item = crate::identity::data::PartitionId>,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        self.basis
            .demand_regions_head(execution, partitions)
            .map_err(Into::into)
    }
    fn admitted_head_region(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.basis.head_region(id)
    }
    fn demand_regions_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partitions: impl IntoIterator<Item = crate::identity::data::PartitionId>,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        self.basis
            .demand_regions_full(execution, partitions)
            .map_err(Into::into)
    }

    fn selected_storage(&self) -> Option<&dyn crate::storage::overlay::PartitionAccess> {
        Some(&self.basis)
    }
    fn selected_schema_registry(&self) -> Option<&crate::schema::data::RelationalSchemaRegistry> {
        Some(self.schema())
    }

    fn try_for_each_selected_partition<E>(
        &self,
        visit: impl FnMut(crate::identity::data::PartitionId) -> Result<(), E>,
    ) -> Result<(), E> {
        self.basis.root().try_for_each_partition_id(visit)
    }

    fn selected_entity_ids(
        &self,
        kind: crate::identity::data::KindId,
        partition: Option<crate::identity::data::PartitionId>,
    ) -> impl Iterator<Item = crate::identity::data::EntityId> + '_ {
        self.basis.root().entity_ids_of_kind(kind, partition)
    }

    fn selected_relation_ids(
        &self,
        kind: crate::identity::data::KindId,
        partition: Option<crate::identity::data::PartitionId>,
    ) -> impl Iterator<Item = crate::identity::data::RelationId> + '_ {
        self.basis.root().relation_ids_of_kind(kind, partition)
    }

    fn retained_head_region(
        &self,
        partition: crate::identity::data::PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.basis.root().accepted_head_region_view(partition)
    }
    fn is_exact_basis(&self) -> bool {
        true
    }
    fn version_id(&self) -> crate::identity::data::VersionId {
        self.basis.version_id()
    }
}

impl<'a, Snapshot> From<&'a RelationalPinnedRead<Snapshot>>
    for super::super::projection::pinned::PinnedProjectionSource<'a>
{
    fn from(read: &'a RelationalPinnedRead<Snapshot>) -> Self {
        Self {
            resident: read.resident_basis(),
            runtime_identity: read.runtime_instance_id(),
            current_version: read.pinned_current_version(),
            instrumentation: read.pinned_instrumentation(),
        }
    }
}

impl<Snapshot> RelationalPinnedRead<Snapshot> {
    /// Project the exact retained physical root without a snapshot claim.
    pub fn project_read(&self) -> super::super::projection::VisibilityProjectionView {
        super::super::projection::VisibilityProjectionView::from_pinned(self)
    }
}

impl RelationalPinnedRead {
    /// Project this exact snapshot. No installed owner is borrowed or consulted.
    pub fn project_snapshot(
        &self,
        handle: &SnapshotHandle,
    ) -> Option<super::super::projection::VisibilityProjectionView> {
        (handle == self.snapshot()).then(|| self.project_read())
    }
}
