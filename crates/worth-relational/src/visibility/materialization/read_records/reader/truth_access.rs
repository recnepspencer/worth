use super::*;
use crate::storage::data::RecordLifecycleState;

impl<'runtime> VisibilityReadContext<'runtime> {
    pub fn visible_entity_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: crate::identity::data::EntityId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<Option<EntityReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        crate::visibility::materialization::read_records::read_work::point(
            execution,
            || {
                self.authoritative_entity_record_at_version(entity_id, version_id)
                    .filter(|record| record.lifecycle == RecordLifecycleState::Live)
            },
            |record| {
                record
                    .as_ref()
                    .map_or(0, super::super::read_footprint::entity)
            },
        )
    }

    pub fn visible_relation_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relation_id: crate::identity::data::RelationId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<Option<RelationReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        crate::visibility::materialization::read_records::read_work::point(
            execution,
            || {
                self.authoritative_relation_record_at_version(relation_id, version_id)
                    .filter(|record| record.lifecycle == RecordLifecycleState::Live)
            },
            |record| {
                record
                    .as_ref()
                    .map_or(0, super::super::read_footprint::relation)
            },
        )
    }

    pub fn visible_entities_of_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        crate::visibility::materialization::read_records::read_work::metered(
            execution,
            |context| {
                let state = self.runtime.storage_access().current_edition();
                let mut records = Vec::new();
                for partition_id in state.partition_ids() {
                    records.extend(self.visible_entities_of_kind_in_partition_from_state(
                        &state,
                        partition_id,
                        kind_id,
                        version_id,
                        &mut |units| context.checkpoint(units),
                    )?);
                }
                Ok::<_, super::EntityKindTruthReadLimitExceeded>(records)
            },
            |result| {
                result
                    .as_ref()
                    .map_or(0, super::super::read_footprint::entities)
            },
        )?
        .map_err(|_| crate::execution::RelationalExecutionDenialCause::WorkExhausted)
    }

    pub fn visible_entities_of_kind_in_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition_id: crate::identity::data::PartitionId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        crate::visibility::materialization::read_records::read_work::metered(
            execution,
            |context| {
                let state = self.runtime.storage_access().current_edition();
                self.visible_entities_of_kind_in_partition_from_state(
                    &state,
                    partition_id,
                    kind_id,
                    version_id,
                    &mut |units| context.checkpoint(units),
                )
            },
            |result| {
                result
                    .as_ref()
                    .map_or(0, super::super::read_footprint::entities)
            },
        )?
        .map_err(|_| crate::execution::RelationalExecutionDenialCause::WorkExhausted)
    }

    pub fn visible_relations_of_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<Vec<RelationReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        crate::visibility::materialization::read_records::read_work::metered(
            execution,
            |context| {
                let state = self.runtime.storage_access().current_edition();
                let mut records = Vec::new();
                for partition_id in state.partition_ids() {
                    records.extend(self.visible_relations_of_kind_in_partition_from_state(
                        &state,
                        partition_id,
                        kind_id,
                        version_id,
                        &mut |units| context.checkpoint(units),
                    )?);
                }
                super::truth_record_access::sort_authoritative_relation_records(&mut records);
                Ok::<_, super::RelationKindTruthReadDenial>(records)
            },
            |result| {
                result
                    .as_ref()
                    .map_or(0, super::super::read_footprint::relations)
            },
        )?
        .map_err(|_| crate::execution::RelationalExecutionDenialCause::WorkExhausted)
    }

    pub fn visible_relations_of_kind_in_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition_id: crate::identity::data::PartitionId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<Vec<RelationReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        crate::visibility::materialization::read_records::read_work::metered(
            execution,
            |context| {
                let state = self.runtime.storage_access().current_edition();
                let mut records = self.visible_relations_of_kind_in_partition_from_state(
                    &state,
                    partition_id,
                    kind_id,
                    version_id,
                    &mut |units| context.checkpoint(units),
                )?;
                super::truth_record_access::sort_authoritative_relation_records(&mut records);
                Ok::<_, super::RelationKindTruthReadDenial>(records)
            },
            |result| {
                result
                    .as_ref()
                    .map_or(0, super::super::read_footprint::relations)
            },
        )?
        .map_err(|_| crate::execution::RelationalExecutionDenialCause::WorkExhausted)
    }
    pub fn entity_aspects_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: crate::identity::data::EntityId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<Option<Vec<AspectKey>>, crate::execution::RelationalExecutionDenialCause> {
        crate::visibility::materialization::read_records::read_request::checked(execution, || {
            let state = self.runtime.storage_access().current_edition();
            let record =
                self.authoritative_entity_record_for_id_at_version(&state, entity_id, version_id)?;
            Some(super::aspect_catalog::declared_aspects_for_entity_kind(
                self.runtime,
                record.kind.kind_id,
            ))
        })
    }

    pub fn relation_aspects_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relation_id: crate::identity::data::RelationId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<Option<Vec<AspectKey>>, crate::execution::RelationalExecutionDenialCause> {
        crate::visibility::materialization::read_records::read_request::checked(execution, || {
            let state = self.runtime.storage_access().current_edition();
            let record = self.authoritative_relation_record_for_id_at_version(
                &state,
                relation_id,
                version_id,
            )?;
            Some(super::aspect_catalog::declared_aspects_for_relation_kind(
                self.runtime,
                record.kind.kind_id,
            ))
        })
    }
}

pub(super) fn authoritative_entity_records_are_canonical(records: &[EntityReadRecord]) -> bool {
    records.windows(2).all(|window| {
        let left = &window[0];
        let right = &window[1];
        (
            left.entity_id.partition_id.0,
            left.entity_id.local_slot.0,
            left.entity_id.generation,
        ) <= (
            right.entity_id.partition_id.0,
            right.entity_id.local_slot.0,
            right.entity_id.generation,
        )
    })
}
