use super::{
    EntityProjectionRecord, EntityRecordProjection, ProjectionAspectScope,
    RelationProjectionRecord, RelationRecordProjection, VisibilityProjectionView,
};
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};

impl VisibilityProjectionView<'_> {
    pub fn entities<T: EntityRecordProjection>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Vec<T>, crate::execution::RelationalExecutionDenialCause> {
        Ok({
            let projection_scope = self.assert_entity_projection_contract::<T>();
            self.authoritative_entity_records(execution, T::KIND)?
                .into_iter()
                .filter_map(|record| {
                    T::from_record(EntityProjectionRecord::new(&record, &projection_scope))
                })
                .collect()
        })
    }

    pub fn entities_in<T: EntityRecordProjection>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition_id: PartitionId,
    ) -> Result<Vec<T>, crate::execution::RelationalExecutionDenialCause> {
        Ok({
            let projection_scope = self.assert_entity_projection_contract::<T>();
            self.authoritative_entity_records_in(execution, partition_id, T::KIND)?
                .into_iter()
                .filter_map(|record| {
                    T::from_record(EntityProjectionRecord::new(&record, &projection_scope))
                })
                .collect()
        })
    }

    pub fn entity<T: EntityRecordProjection>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
    ) -> Result<Option<T>, crate::execution::RelationalExecutionDenialCause> {
        Ok({
            let projection_scope = self.assert_entity_projection_contract::<T>();
            self.project_entity_record(execution, entity_id, &projection_scope, T::from_record)?
        })
    }

    pub fn entity_records_with_projection_scope<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: KindId,
        projection_scope: ProjectionAspectScope,
        mut project: impl FnMut(EntityProjectionRecord<'_>) -> Option<T>,
    ) -> Result<Vec<T>, crate::execution::RelationalExecutionDenialCause> {
        Ok({
            self.assert_entity_projection_scope(kind_id, &projection_scope);
            self.authoritative_entity_records(execution, kind_id)?
                .into_iter()
                .filter_map(|record| {
                    project(EntityProjectionRecord::new(&record, &projection_scope))
                })
                .collect()
        })
    }

    pub fn entity_record_with_projection_scope<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        projection_scope: ProjectionAspectScope,
        mut project: impl FnMut(EntityProjectionRecord<'_>) -> Option<T>,
    ) -> Result<Option<T>, crate::execution::RelationalExecutionDenialCause> {
        Ok({
            self.project_entity_record(execution, entity_id, &projection_scope, |record| {
                self.assert_entity_projection_scope(record.kind_id(), &projection_scope);
                project(record)
            })?
        })
    }

    pub(crate) fn entity_record_of_expected_kind_with_projection_scope<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        expected_kind_id: KindId,
        projection_scope: ProjectionAspectScope,
        mut project: impl FnMut(EntityProjectionRecord<'_>) -> Option<T>,
    ) -> Result<Result<Option<T>, KindId>, crate::execution::RelationalExecutionDenialCause> {
        Ok({
            self.project_entity_record(execution, entity_id, &projection_scope, |record| {
                Some(if record.kind_id() != expected_kind_id {
                    Err(record.kind_id())
                } else {
                    self.assert_entity_projection_scope(expected_kind_id, &projection_scope);
                    Ok(project(record))
                })
            })?
            .unwrap_or(Ok(None))
        })
    }

    pub fn relations<T: RelationRecordProjection>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Vec<T>, crate::execution::RelationalExecutionDenialCause> {
        Ok({
            let projection_scope = self.assert_relation_projection_contract::<T>();
            self.authoritative_relation_records(execution, T::KIND)?
                .into_iter()
                .filter_map(|record| {
                    T::from_record(RelationProjectionRecord::new(&record, &projection_scope))
                })
                .collect()
        })
    }

    pub fn relations_in<T: RelationRecordProjection>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition_id: PartitionId,
    ) -> Result<Vec<T>, crate::execution::RelationalExecutionDenialCause> {
        Ok({
            let projection_scope = self.assert_relation_projection_contract::<T>();
            self.authoritative_relation_records_in(execution, partition_id, T::KIND)?
                .into_iter()
                .filter_map(|record| {
                    T::from_record(RelationProjectionRecord::new(&record, &projection_scope))
                })
                .collect()
        })
    }

    pub fn relation<T: RelationRecordProjection>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relation_id: RelationId,
    ) -> Result<Option<T>, crate::execution::RelationalExecutionDenialCause> {
        Ok({
            let projection_scope = self.assert_relation_projection_contract::<T>();
            self.authoritative_relation_record(execution, relation_id)?
                .and_then(|record| {
                    T::from_record(RelationProjectionRecord::new(&record, &projection_scope))
                })
        })
    }

    pub fn relation_records_with_projection_scope<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: KindId,
        projection_scope: ProjectionAspectScope,
        mut project: impl FnMut(RelationProjectionRecord<'_>) -> Option<T>,
    ) -> Result<Vec<T>, crate::execution::RelationalExecutionDenialCause> {
        Ok({
            self.assert_relation_projection_scope(kind_id, &projection_scope);
            self.authoritative_relation_records(execution, kind_id)?
                .into_iter()
                .filter_map(|record| {
                    project(RelationProjectionRecord::new(&record, &projection_scope))
                })
                .collect()
        })
    }

    pub fn relation_record_with_projection_scope<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relation_id: RelationId,
        projection_scope: ProjectionAspectScope,
        mut project: impl FnMut(RelationProjectionRecord<'_>) -> Option<T>,
    ) -> Result<Option<T>, crate::execution::RelationalExecutionDenialCause> {
        Ok({
            let Some(record) = self.authoritative_relation_record(execution, relation_id)? else {
                return Ok(None);
            };
            self.assert_relation_projection_scope(record.kind.kind_id, &projection_scope);
            project(RelationProjectionRecord::new(&record, &projection_scope))
        })
    }
}
