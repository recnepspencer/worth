//! Immutable native custody independent of the installed runtime's borrow.
use branch::history::retention::selected_snapshot::VisibilitySnapshotBasis;
use branch::snapshots::data::SnapshotHandle;
use branch::visibility::materialization::read_records::projection::pinned::VisibilityProjectionViewOwnerPort as _;
use branch::visibility::snapshot_states::resident_basis::ResidentVisibilitySnapshotBasis;
use contracts::indexes::data::definition::DerivedIndexDefinition;
use contracts::indexes::data::definition::DerivedIndexGenerationId;
use contracts::indexes::data::definition::DerivedIndexId;
use contracts::query::data::planned_packets::QueryPlanContextId;
use contracts::schema::data::RelationalSchemaRegistry;
use contracts::symbols::data::string_interner::StringInterner;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use std::{collections::BTreeMap, sync::Arc};
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_indexes::internal_port::indexes::access::selected_index::PinnedIndex;

pub mod aggregate_reads;
pub mod candidate;
pub mod derived_source;
pub mod frontier_reads;
pub mod index_lookups;

pub use candidate::RelationalAppliedCandidateReadIdentity;
use contracts::indexes::data::applied_candidate_qualification::CandidateReadPhase;
pub use contracts::indexes::data::applied_candidate_qualification::RelationalAppliedCandidateReadQualification;

/// An exact root, its schema/frontiers/revisions, isolated dictionary namespace,
/// and selected index handles. Reads demand only reached regions and symbol
/// pages. Cloning establishes another immutable observer; it copies
/// neither rows nor an index. No runtime, request, or registry guard is retained.
/// Snapshot-backed pins carry their handle. Observation-backed pins carry `()`;
/// only snapshot-backed pins expose snapshot/plan doors, by type.
#[derive(Clone, Debug)]
pub struct RelationalPinnedRead<Snapshot = SnapshotHandle> {
    pub(super) applied_changes: Option<candidate::changes::SharedAppliedReadChanges>,
    pub(super) current_version: contracts::identity::data::primitives::VersionId,
    pub(super) instrumentation:
        branch::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
    pub(super) snapshot: Snapshot,
    pub(super) basis: ResidentVisibilitySnapshotBasis,
    pub(super) context: QueryPlanContextId,
    pub(super) indexes: BTreeMap<DerivedIndexId, PinnedIndex>,
}

pub use branch::visibility::materialization::read_records::reader::pin_denial::RelationalPinnedReadAdmissionDenial;

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
        self.basis.projection_root().schema_authority().registry()
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
        symbol: contracts::symbols::data::symbol::Symbol,
        inspect: impl FnOnce(Option<&contracts::symbols::data::stored_text::StoredSymbolText>) -> T,
    ) -> Result<T, contracts::execution::denial_cause::RelationalExecutionDenialCause> {
        contracts::execution::request_custody::check_request(execution)?;
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
    ) -> Option<&worth_relational_indexes::internal_port::indexes::authority::maintenance::applied_field::AppliedFieldImage>{
        self.applied_changes
            .as_ref()
            .map(|changes| changes.fields.as_ref())
    }
    pub(crate) fn candidate_aggregate(
        &self,
    ) -> Option<&worth_relational_mutation::internal_port::aggregate_identity::candidate_read::AggregateCandidateRead>{
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
    pub(crate) fn pinned_current_version(
        &self,
    ) -> contracts::identity::data::primitives::VersionId {
        self.current_version
    }
    pub(crate) fn pinned_instrumentation(
        &self,
    ) -> &branch::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation
    {
        &self.instrumentation
    }
}

impl<Snapshot> branch::visibility::materialization::read_records::projection::streamed_basis_reads::ProjectionRecordSource for RelationalPinnedRead<Snapshot> {
    // A pin retains the exact root and its dictionary. Each operation admits
    // only the regions it reaches before borrowing their materialized rows.
    fn demand_all_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), contracts::execution::denial_cause::RelationalExecutionDenialCause> {
        self.basis
            .demand_regions_full(execution, self.basis.projection_root().partition_ids_iter())
            .map_err(Into::into)
    }
    fn demand_regions_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partitions: impl IntoIterator<Item = contracts::identity::data::primitives::PartitionId>,
    ) -> Result<(), contracts::execution::denial_cause::RelationalExecutionDenialCause> {
        self.basis
            .demand_regions_head(execution, partitions)
            .map_err(Into::into)
    }
    fn admitted_head_region(
        &self,
        id: contracts::identity::data::primitives::PartitionId,
    ) -> Option<worth_relational_storage::internal_port::storage::residency::views::RelationalRegionHeadView<'_>> {
        self.basis.head_region(id)
    }
    fn demand_regions_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partitions: impl IntoIterator<Item = contracts::identity::data::primitives::PartitionId>,
    ) -> Result<(), contracts::execution::denial_cause::RelationalExecutionDenialCause> {
        self.basis
            .demand_regions_full(execution, partitions)
            .map_err(Into::into)
    }

    fn selected_storage(&self) -> Option<&dyn worth_relational_storage::internal_port::storage::overlay::access::PartitionAccess> {
        Some(&self.basis)
    }
    fn selected_schema_registry(&self) -> Option<&contracts::schema::data::RelationalSchemaRegistry> {
        Some(self.schema())
    }

    fn try_for_each_selected_partition<E>(
        &self,
        visit: impl FnMut(contracts::identity::data::primitives::PartitionId) -> Result<(), E>,
    ) -> Result<(), E> {
        self.basis
            .projection_root()
            .try_for_each_partition_id(visit)
    }

    fn selected_entity_ids(
        &self,
        kind: contracts::identity::data::primitives::KindId,
        partition: Option<contracts::identity::data::primitives::PartitionId>,
    ) -> impl Iterator<Item = Result<contracts::identity::data::record_ids::EntityId, worth_relational_storage::internal_port::storage::data::RelationalStorageUnavailable>> + '_ {
        self.basis
            .projection_root()
            .entity_ids_of_kind(kind, partition)
    }

    fn selected_relation_ids(
        &self,
        kind: contracts::identity::data::primitives::KindId,
        partition: Option<contracts::identity::data::primitives::PartitionId>,
    ) -> impl Iterator<Item = Result<contracts::identity::data::record_ids::RelationId, worth_relational_storage::internal_port::storage::data::RelationalStorageUnavailable>> + '_ {
        self.basis
            .projection_root()
            .relation_ids_of_kind(kind, partition)
    }

    fn retained_head_region(
        &self,
        partition: contracts::identity::data::primitives::PartitionId,
    ) -> Option<worth_relational_storage::internal_port::storage::residency::views::RelationalRegionHeadView<'_>> {
        self.basis
            .projection_root()
            .accepted_head_region_view(partition)
    }
    fn is_exact_basis(&self) -> bool {
        true
    }
    fn version_id(&self) -> contracts::identity::data::primitives::VersionId {
        self.basis.version_id()
    }
}

impl<'a, Snapshot> From<&'a RelationalPinnedRead<Snapshot>>
    for branch::visibility::materialization::read_records::projection::pinned::PinnedProjectionSource<'a>
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
    pub fn project_read(
        &self,
    ) -> branch::visibility::materialization::read_records::projection::view::VisibilityProjectionView
    {
        branch::visibility::materialization::read_records::projection::view::VisibilityProjectionView::from_pinned(self)
    }
}

impl RelationalPinnedRead {
    /// Project this exact snapshot. No installed owner is borrowed or consulted.
    pub fn project_snapshot(
        &self,
        handle: &SnapshotHandle,
    ) -> Option<branch::visibility::materialization::read_records::projection::view::VisibilityProjectionView>{
        (handle == self.snapshot()).then(|| self.project_read())
    }
}

#[doc(hidden)]
pub trait RelationalPinnedReadOwnerPort<Snapshot>: Sized {
    fn capture_exact_pin(
        current_version: contracts::identity::data::primitives::VersionId,
        instrumentation: branch::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
        snapshot: Snapshot,
        basis: ResidentVisibilitySnapshotBasis,
        context: QueryPlanContextId,
        indexes: BTreeMap<DerivedIndexId, PinnedIndex>,
    ) -> Self;

    fn resident_basis(&self) -> &ResidentVisibilitySnapshotBasis;
}

impl<Snapshot> RelationalPinnedReadOwnerPort<Snapshot> for RelationalPinnedRead<Snapshot> {
    fn capture_exact_pin(
        current_version: contracts::identity::data::primitives::VersionId,
        instrumentation: branch::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
        snapshot: Snapshot,
        basis: ResidentVisibilitySnapshotBasis,
        context: QueryPlanContextId,
        indexes: BTreeMap<DerivedIndexId, PinnedIndex>,
    ) -> Self {
        Self {
            current_version,
            instrumentation,
            snapshot,
            basis,
            context,
            indexes,
            applied_changes: None,
        }
    }

    fn resident_basis(&self) -> &ResidentVisibilitySnapshotBasis {
        self.resident_basis()
    }
}

#[doc(hidden)]
pub trait RelationalPinnedReadFixturePort {
    fn indexes_for_test(&self) -> &BTreeMap<DerivedIndexId, PinnedIndex>;
}
impl<Snapshot> RelationalPinnedReadFixturePort for RelationalPinnedRead<Snapshot> {
    fn indexes_for_test(&self) -> &BTreeMap<DerivedIndexId, PinnedIndex> {
        &self.indexes
    }
}
