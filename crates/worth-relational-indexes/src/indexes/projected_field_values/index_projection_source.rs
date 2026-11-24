use crate::branch::visibility::materialization::read_records::projection::basis_reads::VisibilityProjectionViewOwnerPort as _;
use crate::branch::visibility::materialization::read_records::projection::streamed_basis_reads::ProjectionRecordSource;
use crate::branch::visibility::materialization::read_records::projection::view::{
    VisibilityProjectionView, VisibilityProjectionViewOwnerPort as _,
};
use crate::contracts::identity::data::primitives::KindId;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::schema::data::aspect_contracts::lowered_plan::{
    AspectContractPlanCatalog, LoweredAspectContractPlan,
};
use crate::contracts::schema::data::SchemaVersionId;
use crate::contracts::storage::data::read_records::{EntityReadRecord, RelationReadRecord};

/// Schema-qualified storage projection used to derive or certify an index.
///
/// Construction is private to this module so a caller cannot pair records
/// with an ambient or unrelated schema authority.
pub struct IndexProjectionSource<'view> {
    projection: &'view VisibilityProjectionView,
    reconstruction:
        Option<&'view crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'view>>,
}

impl<'view> IndexProjectionSource<'view> {
    pub(in crate::internal_port::indexes) fn reconstructing<'lane>(
        &'lane self,
        lane: &'lane crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    ) -> IndexProjectionSource<'lane> {
        IndexProjectionSource {
            projection: self.projection,
            reconstruction: Some(lane),
        }
    }

    pub(in crate::internal_port::indexes) fn selected(
        projection: &'view VisibilityProjectionView,
    ) -> Self {
        Self {
            projection,
            reconstruction: None,
        }
    }
    pub fn exact(projection: &'view VisibilityProjectionView) -> Option<Self> {
        projection.is_exact_basis().then_some(Self {
            projection,
            reconstruction: None,
        })
    }

    pub fn historical(projection: &'view VisibilityProjectionView) -> Option<Self> {
        (!projection.is_exact_basis()).then_some(Self {
            projection,
            reconstruction: None,
        })
    }

    pub(in crate::internal_port::indexes) fn schema_version(&self) -> Option<SchemaVersionId> {
        self.projection
            .selected_schema()
            .map(|schema| schema.version)
    }

    pub(in crate::internal_port::indexes) fn schema_authority_digest(&self) -> Option<[u8; 32]> {
        self.projection
            .selected_schema()
            .map(|schema| schema.authority_digest)
    }

    pub(in crate::internal_port::indexes) fn aspect_plans(
        &self,
    ) -> Option<&AspectContractPlanCatalog> {
        self.projection.selected_schema().map(|schema| schema.plans)
    }

    pub(in crate::internal_port::indexes) fn entity_aspect_plan(
        &self,
        kind_id: KindId,
    ) -> Option<&LoweredAspectContractPlan> {
        self.aspect_plans()?.entity_plans.get(&kind_id)
    }

    /// Cost: the selected endpoint's adjacency, never the complete relation kind.
    pub(in crate::internal_port::indexes) fn relations_for_endpoint(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
        kind: KindId,
        outgoing: bool,
    ) -> Result<
        Vec<RelationReadRecord>,
        crate::contracts::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        self.projection
            .bounded_index_relations_for_frontier(
                execution,
                &std::collections::BTreeSet::from([entity]),
                kind,
                outgoing,
                None,
            )?
            .map(|read| read.into_records())
            .map_err(|_| crate::contracts::execution::denial_cause::RelationalExecutionDenialCause::ResultCapacityExceeded)
    }

    pub(in crate::internal_port::indexes) fn for_each_relation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: KindId,
        mut visit: impl FnMut(
            &RelationReadRecord,
        ) -> Result<
            (),
            crate::contracts::execution::denial_cause::RelationalExecutionDenialCause,
        >,
    ) -> Result<(), crate::contracts::execution::denial_cause::RelationalExecutionDenialCause> {
        for record in self
            .projection
            .authoritative_relation_records(execution, kind_id)?
        {
            visit(&record)?;
        }
        Ok(())
    }

    pub(in crate::internal_port::indexes) fn try_for_each_entity<E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: KindId,
        before_candidate: impl FnMut(u64, u64) -> Result<(), E>,
        visit: impl FnMut(&EntityReadRecord) -> Result<(), E>,
    ) -> Result<(), crate::branch::visibility::materialization::read_records::read_stop::ReadStop<E>>
    {
        self.projection.try_for_each_entity_record(
            execution,
            Some(kind_id),
            before_candidate,
            visit,
        )
    }

    pub(in crate::internal_port::indexes) fn try_for_each_relation<E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: KindId,
        before_candidate: impl FnMut(u64, u64) -> Result<(), E>,
        visit: impl FnMut(&RelationReadRecord) -> Result<(), E>,
    ) -> Result<(), crate::branch::visibility::materialization::read_records::read_stop::ReadStop<E>>
    {
        self.projection.try_for_each_relation_record(
            execution,
            Some(kind_id),
            before_candidate,
            visit,
        )
    }

    pub fn with_entity<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        inspect: impl FnOnce(&EntityReadRecord) -> T,
    ) -> Result<Option<T>, crate::contracts::execution::denial_cause::RelationalExecutionDenialCause>
    {
        Ok(self
            .projection
            .authoritative_entity_record(execution, entity_id)?
            .as_ref()
            .map(inspect))
    }

    pub(in crate::internal_port::indexes) fn with_entity_checked<T, E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        before: impl FnMut(u64, u64) -> Result<(), E>,
        inspect: impl FnOnce(&EntityReadRecord) -> T,
    ) -> Result<
        Option<T>,
        crate::branch::visibility::materialization::read_records::read_stop::ReadStop<E>,
    > {
        self.projection
            .with_entity_record(execution, entity_id, before, inspect)
    }

    pub fn with_relation<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relation_id: RelationId,
        inspect: impl FnOnce(&RelationReadRecord) -> T,
    ) -> Result<Option<T>, crate::contracts::execution::denial_cause::RelationalExecutionDenialCause>
    {
        Ok(self
            .projection
            .authoritative_relation_record(execution, relation_id)?
            .as_ref()
            .map(inspect))
    }
}

pub mod native_fields;
