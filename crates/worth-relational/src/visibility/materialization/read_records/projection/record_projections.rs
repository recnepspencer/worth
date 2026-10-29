use super::{
    EntityProjectionRecord, EntityRecordProjection, ProjectionAspectScope,
    RelationProjectionRecord, RelationRecordProjection, VisibilityProjectionView,
};
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};

impl VisibilityProjectionView {
    pub fn entities<T: EntityRecordProjection>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Vec<T>, crate::execution::RelationalExecutionDenialCause> {
        Ok({
            let projection_scope = self.admit_entity_projection_contract::<T>()?;
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
            let projection_scope = self.admit_entity_projection_contract::<T>()?;
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
            let projection_scope = self.admit_entity_projection_contract::<T>()?;
            self.project_entity_record(execution, entity_id, &projection_scope, |record| {
                Ok(T::from_record(record))
            })?
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
            self.admit_entity_projection_scope(kind_id, &projection_scope)?;
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
        let mut slot = None;
        let produced = self.entity_record_with_projection_scope_erased(
            execution,
            entity_id,
            &projection_scope,
            &mut |record| match project(record) {
                Some(value) => {
                    slot = Some(value);
                    true
                }
                None => false,
            },
        )?;
        Ok(if produced { slot } else { None })
    }

    #[inline(never)]
    pub(super) fn entity_record_with_projection_scope_erased(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        projection_scope: &ProjectionAspectScope,
        project: &mut dyn for<'a> FnMut(EntityProjectionRecord<'a>) -> bool,
    ) -> Result<bool, crate::execution::RelationalExecutionDenialCause> {
        self.project_entity_record_erased(execution, entity_id, projection_scope, &mut |record| {
            self.admit_entity_projection_scope(record.kind_id(), projection_scope)?;
            Ok(project(record))
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
        let mut slot = None;
        let outcome = self.entity_record_of_expected_kind_with_projection_scope_erased(
            execution,
            entity_id,
            expected_kind_id,
            &projection_scope,
            &mut |record| match project(record) {
                Some(value) => {
                    slot = Some(value);
                    true
                }
                None => false,
            },
        )?;
        Ok(match outcome {
            Err(kind_id) => Err(kind_id),
            Ok(true) => Ok(slot),
            Ok(false) => Ok(None),
        })
    }

    #[inline(never)]
    fn entity_record_of_expected_kind_with_projection_scope_erased(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        expected_kind_id: KindId,
        projection_scope: &ProjectionAspectScope,
        project: &mut dyn for<'a> FnMut(EntityProjectionRecord<'a>) -> bool,
    ) -> Result<Result<bool, KindId>, crate::execution::RelationalExecutionDenialCause> {
        let mut mismatched_kind = None;
        let produced = self.project_entity_record_erased(
            execution,
            entity_id,
            projection_scope,
            &mut |record| {
                if record.kind_id() != expected_kind_id {
                    mismatched_kind = Some(record.kind_id());
                    return Ok(false);
                }
                self.admit_entity_projection_scope(expected_kind_id, projection_scope)?;
                Ok(project(record))
            },
        )?;
        // No record leaves `produced` false and does not set a kind. A closure
        // `None` is the same outer `Ok(None)`, distinct from a kind mismatch.
        Ok(match mismatched_kind {
            Some(kind_id) => Err(kind_id),
            None => Ok(produced),
        })
    }

    pub fn relations<T: RelationRecordProjection>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Vec<T>, crate::execution::RelationalExecutionDenialCause> {
        Ok({
            let projection_scope = self.admit_relation_projection_contract::<T>()?;
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
            let projection_scope = self.admit_relation_projection_contract::<T>()?;
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
            let projection_scope = self.admit_relation_projection_contract::<T>()?;
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
            self.admit_relation_projection_scope(kind_id, &projection_scope)?;
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
        let mut slot = None;
        let produced = self.relation_record_with_projection_scope_erased(
            execution,
            relation_id,
            &projection_scope,
            &mut |record| match project(record) {
                Some(value) => {
                    slot = Some(value);
                    true
                }
                None => false,
            },
        )?;
        Ok(if produced { slot } else { None })
    }

    #[inline(never)]
    fn relation_record_with_projection_scope_erased(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relation_id: RelationId,
        projection_scope: &ProjectionAspectScope,
        project: &mut dyn for<'a> FnMut(RelationProjectionRecord<'a>) -> bool,
    ) -> Result<bool, crate::execution::RelationalExecutionDenialCause> {
        let Some(record) = self.authoritative_relation_record(execution, relation_id)? else {
            return Ok(false);
        };
        self.admit_relation_projection_scope(record.kind.kind_id, projection_scope)?;
        Ok(project(RelationProjectionRecord::new(
            &record,
            projection_scope,
        )))
    }
}
