//! Exact-basis single-entity projection borrows the selected root's values.

use crate::identity::data::EntityId;
use crate::visibility::snapshot_states::SnapshotStateBasis;

use super::{EntityProjectionRecord, ProjectionAspectScope, VisibilityProjectionView};

impl VisibilityProjectionView {
    pub(super) fn project_entity_record<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
        scope: &ProjectionAspectScope,
        mut project: impl FnMut(EntityProjectionRecord<'_>) -> Option<T>,
    ) -> Result<Option<T>, crate::execution::RelationalExecutionDenialCause> {
        let SnapshotStateBasis::Exact(_) = &self.basis else {
            // Historical reconstruction retains its own version-selection rules.
            let Some(record) = self.authoritative_entity_record(execution, entity)? else {
                return Ok(None);
            };
            return Ok(project(EntityProjectionRecord::new(&record, scope)));
        };
        self.demand_regions_head(execution, [entity.partition_id])?;
        let resolved = super::super::read_work::point(
            execution,
            || self.selected_entity_projection(entity),
            |_| 0,
        )?;
        Ok(resolved.and_then(|selected| selected.project(scope, &mut project)))
    }

    /// Borrow one exact-root entity after its caller has admitted the keyed
    /// read. The caller owns that charge; request liveness is still checked
    /// before touching truth. Historical reconstruction uses the paid reader.
    pub fn entity_record_with_projection_scope_admitted<T, Stop>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
        scope: ProjectionAspectScope,
        mut admit_work: impl FnMut(u64) -> Result<(), Stop>,
        mut project: impl FnMut(EntityProjectionRecord<'_>) -> Option<T>,
    ) -> Result<Result<Option<T>, Stop>, crate::execution::RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)?;
        if !self.is_exact_basis() {
            return self
                .entity_record_with_projection_scope(execution, entity, scope, project)
                .map(Ok);
        }
        self.demand_regions_head(execution, [entity.partition_id])?;
        if let Err(stop) = admit_work(super::super::read_prices::KEYED_RECORD) {
            return Ok(Err(stop));
        }
        Ok(Ok(self.selected_entity_projection(entity).and_then(
            |selected| {
                self.assert_entity_projection_scope(selected.kind.kind_id, &scope);
                selected.project(&scope, &mut project)
            },
        )))
    }

    fn selected_entity_projection(&self, entity: EntityId) -> Option<SelectedEntityProjection<'_>> {
        let root = self.basis.root()?;
        let partition = root.admitted_head_region(entity.partition_id)?;
        let slot = partition.entity_arena().get_slot(entity.slot_index())?;
        if !slot.is_live()
            || (!entity.generation.is_zero() && slot.generation() != entity.generation_value())
        {
            return None;
        }
        let kind = root
            .schema_authority()
            .registry()
            .resolve_entity(slot.kind_id()?)
            .ok()?;
        let actual = EntityId::new(
            entity.partition_id,
            entity.local_slot_value(),
            slot.generation(),
        );
        let created = partition
            .entity_arena()
            .created_at_for_slot(entity.slot_index())
            .expect("visible entity slot has creation metadata");
        Some(SelectedEntityProjection {
            actual,
            slot,
            kind,
            created,
        })
    }
}

struct SelectedEntityProjection<'a> {
    actual: EntityId,
    slot: crate::storage::substrate::SlotView<'a, crate::storage::substrate::EntityRecordKind>,
    kind: crate::schema::data::KindResolution,
    created: crate::identity::data::VersionId,
}

impl SelectedEntityProjection<'_> {
    fn project<T>(
        self,
        scope: &ProjectionAspectScope,
        project: &mut impl FnMut(EntityProjectionRecord<'_>) -> Option<T>,
    ) -> Option<T> {
        project(EntityProjectionRecord::from_slot(
            self.actual,
            &self.slot,
            &self.kind,
            self.created,
            scope,
        ))
    }
}

#[cfg(test)]
mod tests;
