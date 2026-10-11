use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};

use super::exact_basis_reads::ExactProjectionReader;
use super::historical_basis_reads::HistoricalProjectionReader;
use super::VisibilityProjectionView;

impl VisibilityProjectionView<'_> {
    pub(crate) fn authoritative_entity_records(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: KindId,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        crate::visibility::materialization::read_records::read_work::metered(
            execution,
            |context| match &self.basis {
                crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis) => {
                    Ok(ExactProjectionReader::new(self, basis.root()).entity_records(kind_id))
                }
                crate::visibility::snapshot_states::SnapshotStateBasis::Historical(basis) => {
                    HistoricalProjectionReader::new(self, basis)
                        .entity_records(kind_id, &mut |units| context.checkpoint(units))
                }
            },
            |result| {
                result
                    .as_ref()
                    .map_or(0, super::super::read_footprint::entities)
            },
        )?
        .map_err(|_| crate::execution::RelationalExecutionDenialCause::WorkExhausted)
    }

    pub(crate) fn authoritative_entity_record(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
    ) -> Result<Option<EntityReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        crate::visibility::materialization::read_records::read_request::checked(execution, || {
            match &self.basis {
                crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis) => {
                    ExactProjectionReader::new(self, basis.root()).entity_record(entity_id)
                }
                crate::visibility::snapshot_states::SnapshotStateBasis::Historical(basis) => {
                    HistoricalProjectionReader::new(self, basis).entity_record(entity_id)
                }
            }
        })
    }

    pub(crate) fn all_authoritative_entity_records(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        crate::visibility::materialization::read_records::read_request::checked(execution, || {
            match &self.basis {
                crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis) => {
                    ExactProjectionReader::new(self, basis.root()).all_entity_records()
                }
                crate::visibility::snapshot_states::SnapshotStateBasis::Historical(basis) => {
                    HistoricalProjectionReader::new(self, basis).all_entity_records()
                }
            }
        })
    }

    pub(crate) fn authoritative_entity_records_in(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition_id: PartitionId,
        kind_id: KindId,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        crate::visibility::materialization::read_records::read_work::metered(
            execution,
            |context| match &self.basis {
                crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis) => {
                    Ok(ExactProjectionReader::new(self, basis.root())
                        .entity_records_in(partition_id, kind_id))
                }
                crate::visibility::snapshot_states::SnapshotStateBasis::Historical(basis) => {
                    HistoricalProjectionReader::new(self, basis).entity_records_in(
                        partition_id,
                        kind_id,
                        &mut |units| context.checkpoint(units),
                    )
                }
            },
            |result| {
                result
                    .as_ref()
                    .map_or(0, super::super::read_footprint::entities)
            },
        )?
        .map_err(|_| crate::execution::RelationalExecutionDenialCause::WorkExhausted)
    }

    pub(crate) fn authoritative_relation_records(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: KindId,
    ) -> Result<Vec<RelationReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        crate::visibility::materialization::read_records::read_work::metered(
            execution,
            |context| match &self.basis {
                crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis) => {
                    Ok(ExactProjectionReader::new(self, basis.root()).relation_records(kind_id))
                }
                crate::visibility::snapshot_states::SnapshotStateBasis::Historical(basis) => {
                    HistoricalProjectionReader::new(self, basis)
                        .relation_records(kind_id, &mut |units| context.checkpoint(units))
                }
            },
            |result| {
                result
                    .as_ref()
                    .map_or(0, super::super::read_footprint::relations)
            },
        )?
        .map_err(|_| crate::execution::RelationalExecutionDenialCause::WorkExhausted)
    }

    pub(crate) fn authoritative_relation_record(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relation_id: RelationId,
    ) -> Result<Option<RelationReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        super::super::read_request::checked(execution, || self.project_relation_record(relation_id))
    }

    pub(crate) fn all_authoritative_relation_records(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Vec<RelationReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        crate::visibility::materialization::read_records::read_request::checked(execution, || {
            match &self.basis {
                crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis) => {
                    ExactProjectionReader::new(self, basis.root()).all_relation_records()
                }
                crate::visibility::snapshot_states::SnapshotStateBasis::Historical(basis) => {
                    HistoricalProjectionReader::new(self, basis).all_relation_records()
                }
            }
        })
    }

    pub(crate) fn authoritative_relation_records_in(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition_id: PartitionId,
        kind_id: KindId,
    ) -> Result<Vec<RelationReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        crate::visibility::materialization::read_records::read_work::metered(
            execution,
            |context| match &self.basis {
                crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis) => {
                    Ok(ExactProjectionReader::new(self, basis.root())
                        .relation_records_in(partition_id, kind_id))
                }
                crate::visibility::snapshot_states::SnapshotStateBasis::Historical(basis) => {
                    HistoricalProjectionReader::new(self, basis).relation_records_in(
                        partition_id,
                        kind_id,
                        &mut |units| context.checkpoint(units),
                    )
                }
            },
            |result| {
                result
                    .as_ref()
                    .map_or(0, super::super::read_footprint::relations)
            },
        )?
        .map_err(|_| crate::execution::RelationalExecutionDenialCause::WorkExhausted)
    }
    pub(super) fn project_relation_record(
        &self,
        relation_id: RelationId,
    ) -> Option<RelationReadRecord> {
        match &self.basis {
            crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis) => {
                ExactProjectionReader::new(self, basis.root()).relation_record(relation_id)
            }
            crate::visibility::snapshot_states::SnapshotStateBasis::Historical(basis) => {
                HistoricalProjectionReader::new(self, basis).relation_record(relation_id)
            }
        }
    }
}
