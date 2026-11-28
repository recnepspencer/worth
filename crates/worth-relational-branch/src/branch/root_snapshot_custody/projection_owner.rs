//! Owner access to VisibilityProjectionView across the Relational crate split.

use crate::branch::root::RelationalBranchRoot;
use crate::execution::RelationalExecutionDenialCause;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::storage::data::read_records::{EntityReadRecord, RelationReadRecord};
use crate::storage::partition::AdjacencyDirection;
use crate::visibility::materialization::read_records::projection::pinned::PinnedProjectionSource;
use crate::visibility::materialization::read_records::projection::view::SelectedProjectionSchema;
use crate::visibility::materialization::read_records::projection::{
    EntityProjectionRecord, ProjectionAspectScope, ProjectionInputs, VisibilityProjectionView,
};
use crate::visibility::materialization::read_records::reader::truth_frontier_adjacency::{
    BoundedFrontierAdjacencyTruthRead, FrontierAdjacencyTruthReadLimitExceeded,
};
use crate::visibility::snapshot_states::{ResidentVisibilitySnapshotBasis, SnapshotStateBasis};
use std::collections::BTreeSet;
use std::sync::Arc;

#[doc(hidden)]
pub trait VisibilityProjectionViewOwnerPort: Sized {
    fn selected_root(&self) -> Option<&RelationalBranchRoot>;
    fn selected_partitions(&self) -> Option<&dyn crate::storage::overlay::PartitionAccess>;
    fn retain_selected_root(&self) -> Option<Arc<RelationalBranchRoot>>;
    fn from_resident_basis(
        resident: &ResidentVisibilitySnapshotBasis,
        runtime_identity: u64,
        current_version: VersionId,
        instrumentation: &RuntimeInstrumentation,
    ) -> Self;
    fn from_pinned<'a>(read: impl Into<PinnedProjectionSource<'a>>) -> Self;
    fn retain_pinning_metadata(
        &self,
    ) -> (
        crate::identity::data::VersionId,
        crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
        crate::symbols::data::StringInterner,
    );
    fn demand_all_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalExecutionDenialCause>;
    fn new<'a>(inputs: impl Into<ProjectionInputs<'a>>, basis: SnapshotStateBasis) -> Self;
    fn from_snapshot(
        projection: ProjectionInputs<'_>,
        directory: crate::visibility::snapshot_states::resolution::SnapshotReadDirectory<'_>,
        handle: &crate::snapshots::data::SnapshotHandle,
    ) -> Option<Self>;
    fn entity_record_of_expected_kind_with_projection_scope<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        expected_kind_id: KindId,
        projection_scope: ProjectionAspectScope,
        project: impl FnMut(EntityProjectionRecord<'_>) -> Option<T>,
    ) -> Result<
        Result<Option<T>, KindId>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    >;
    fn bounded_relations_for_frontier(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_ids: &BTreeSet<EntityId>,
        kind_id: KindId,
        direction: AdjacencyDirection,
        maximum_work_units: Option<usize>,
        include_audit: bool,
    ) -> Result<
        Result<BoundedFrontierAdjacencyTruthRead, FrontierAdjacencyTruthReadLimitExceeded>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    >;
    fn is_from_runtime(&self, runtime_instance_id: u64) -> bool;
    fn selected_schema(&self) -> Option<SelectedProjectionSchema<'_>>;
    fn authoritative_entity_record(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
    ) -> Result<
        Option<EntityReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    >;
    fn authoritative_relation_records(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: KindId,
    ) -> Result<
        Vec<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    >;
    fn authoritative_relation_record(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relation_id: RelationId,
    ) -> Result<
        Option<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    >;
    fn authoritative_entity_records_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: KindId,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::denial_cause::RelationalExecutionDenialCause>;
    fn authoritative_entity_records_in_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition_id: PartitionId,
        kind_id: KindId,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::denial_cause::RelationalExecutionDenialCause>;
    fn authoritative_relation_records_in_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition_id: PartitionId,
        kind_id: KindId,
    ) -> Result<
        Vec<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    >;
    fn retain_exact_basis(&self) -> Option<crate::history::retention::VisibilitySnapshotBasis>;
    fn from_observation(
        inputs: ProjectionInputs<'_>,
        observation: &crate::mvcc::observation::RelationalBranchObservation,
    ) -> Result<Self, crate::branch::basis_denial::RelationalBranchBasisDenial>;
}

impl VisibilityProjectionViewOwnerPort for VisibilityProjectionView {
    fn selected_root(&self) -> Option<&RelationalBranchRoot> {
        self.selected_root()
    }

    fn selected_partitions(&self) -> Option<&dyn crate::storage::overlay::PartitionAccess> {
        self.selected_partitions()
    }

    fn retain_selected_root(&self) -> Option<Arc<RelationalBranchRoot>> {
        self.retain_selected_root()
    }

    fn from_resident_basis(
        resident: &ResidentVisibilitySnapshotBasis,
        runtime_identity: u64,
        current_version: VersionId,
        instrumentation: &RuntimeInstrumentation,
    ) -> Self {
        VisibilityProjectionView::from_resident_basis(
            resident,
            runtime_identity,
            current_version,
            instrumentation,
        )
    }

    fn from_pinned<'a>(read: impl Into<PinnedProjectionSource<'a>>) -> Self {
        VisibilityProjectionView::from_pinned(read)
    }

    fn retain_pinning_metadata(
        &self,
    ) -> (
        crate::identity::data::VersionId,
        crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
        crate::symbols::data::StringInterner,
    ) {
        self.retain_pinning_metadata()
    }

    fn demand_all_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalExecutionDenialCause> {
        self.demand_all_full(execution)
    }

    fn new<'a>(inputs: impl Into<ProjectionInputs<'a>>, basis: SnapshotStateBasis) -> Self {
        VisibilityProjectionView::new(inputs, basis)
    }

    fn from_snapshot(
        projection: ProjectionInputs<'_>,
        directory: crate::visibility::snapshot_states::resolution::SnapshotReadDirectory<'_>,
        handle: &crate::snapshots::data::SnapshotHandle,
    ) -> Option<Self> {
        VisibilityProjectionView::from_snapshot(projection, directory, handle)
    }

    fn entity_record_of_expected_kind_with_projection_scope<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        expected_kind_id: KindId,
        projection_scope: ProjectionAspectScope,
        project: impl FnMut(EntityProjectionRecord<'_>) -> Option<T>,
    ) -> Result<
        Result<Option<T>, KindId>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        self.entity_record_of_expected_kind_with_projection_scope(
            execution,
            entity_id,
            expected_kind_id,
            projection_scope,
            project,
        )
    }

    fn bounded_relations_for_frontier(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_ids: &BTreeSet<EntityId>,
        kind_id: KindId,
        direction: AdjacencyDirection,
        maximum_work_units: Option<usize>,
        include_audit: bool,
    ) -> Result<
        Result<BoundedFrontierAdjacencyTruthRead, FrontierAdjacencyTruthReadLimitExceeded>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        self.bounded_relations_for_frontier(
            execution,
            entity_ids,
            kind_id,
            direction,
            maximum_work_units,
            include_audit,
        )
    }

    fn is_from_runtime(&self, runtime_instance_id: u64) -> bool {
        self.is_from_runtime(runtime_instance_id)
    }

    fn selected_schema(&self) -> Option<SelectedProjectionSchema<'_>> {
        self.selected_schema()
    }

    fn authoritative_entity_record(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
    ) -> Result<
        Option<EntityReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        self.authoritative_entity_record(execution, entity_id)
    }

    fn authoritative_relation_records(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: KindId,
    ) -> Result<
        Vec<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        self.authoritative_relation_records(execution, kind_id)
    }

    fn authoritative_relation_record(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relation_id: RelationId,
    ) -> Result<
        Option<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        self.authoritative_relation_record(execution, relation_id)
    }

    fn authoritative_entity_records_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: KindId,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
        self.authoritative_entity_records_for_test(execution, kind_id)
    }

    fn authoritative_entity_records_in_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition_id: PartitionId,
        kind_id: KindId,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
        self.authoritative_entity_records_in_for_test(execution, partition_id, kind_id)
    }

    fn authoritative_relation_records_in_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition_id: PartitionId,
        kind_id: KindId,
    ) -> Result<
        Vec<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        self.authoritative_relation_records_in_for_test(execution, partition_id, kind_id)
    }

    fn retain_exact_basis(&self) -> Option<crate::history::retention::VisibilitySnapshotBasis> {
        self.retain_exact_basis()
    }

    fn from_observation(
        inputs: ProjectionInputs<'_>,
        observation: &crate::mvcc::observation::RelationalBranchObservation,
    ) -> Result<Self, crate::branch::basis_denial::RelationalBranchBasisDenial> {
        VisibilityProjectionView::from_observation(inputs, observation)
    }
}
