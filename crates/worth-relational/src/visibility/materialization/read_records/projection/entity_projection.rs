//! Exact-basis single-entity projection borrows the selected root's values.

use crate::execution::RelationalExecutionDenialCause;
use crate::identity::data::EntityId;
use crate::visibility::snapshot_states::SnapshotStateBasis;

use super::{EntityProjectionRecord, ProjectionAspectScope, VisibilityProjectionView};

impl VisibilityProjectionView {
    pub(super) fn project_entity_record<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
        scope: &ProjectionAspectScope,
        mut project: impl FnMut(
            EntityProjectionRecord<'_>,
        ) -> Result<Option<T>, RelationalExecutionDenialCause>,
    ) -> Result<Option<T>, RelationalExecutionDenialCause> {
        let mut slot = None;
        let produced = self.project_entity_record_erased(
            execution,
            entity,
            scope,
            &mut |record| match project(record)? {
                Some(value) => {
                    slot = Some(value);
                    Ok(true)
                }
                None => Ok(false),
            },
        )?;
        Ok(if produced { slot } else { None })
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
    ) -> Result<Result<Option<T>, Stop>, RelationalExecutionDenialCause> {
        let mut slot = None;
        let mut stopped = None;
        let outcome = self.entity_record_with_projection_scope_admitted_erased(
            execution,
            entity,
            &scope,
            &mut |cost| match admit_work(cost) {
                Ok(()) => Ok(()),
                Err(stop) => {
                    stopped = Some(stop);
                    Err(())
                }
            },
            &mut |record| match project(record) {
                Some(value) => {
                    slot = Some(value);
                    true
                }
                None => false,
            },
        )?;
        Ok(match outcome {
            AdmittedProjectionOutcome::Stopped => Err(stopped.expect(
                /* Proof: Stopped is reported only after this trampoline stores admit_work's stop. */
                "keyed projection admission stores its stop",
            )),
            AdmittedProjectionOutcome::Finished(true) => Ok(slot),
            AdmittedProjectionOutcome::Finished(false) => Ok(None),
        })
    }

    #[inline(never)]
    pub(super) fn project_entity_record_erased(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
        scope: &ProjectionAspectScope,
        project: &mut dyn for<'a> FnMut(
            EntityProjectionRecord<'a>,
        ) -> Result<bool, RelationalExecutionDenialCause>,
    ) -> Result<bool, RelationalExecutionDenialCause> {
        let SnapshotStateBasis::Exact(_) = &self.basis else {
            // Historical reconstruction retains its own version-selection rules.
            let Some(record) = self.authoritative_entity_record(execution, entity)? else {
                return Ok(false);
            };
            return project(EntityProjectionRecord::new(&record, scope));
        };
        self.demand_regions_head(execution, [entity.partition_id])?;
        let resolved = super::super::read_work::point(
            execution,
            || self.selected_entity_projection(entity),
            |_| 0,
        )?;
        let Some(selected) = resolved else {
            return Ok(false);
        };
        selected.project(scope, project)
    }

    #[inline(never)]
    fn entity_record_with_projection_scope_admitted_erased(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
        scope: &ProjectionAspectScope,
        admit_work: &mut dyn FnMut(u64) -> Result<(), ()>,
        project: &mut dyn for<'a> FnMut(EntityProjectionRecord<'a>) -> bool,
    ) -> Result<AdmittedProjectionOutcome, RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)?;
        if !self.is_exact_basis() {
            return self
                .entity_record_with_projection_scope_erased(execution, entity, scope, project)
                .map(AdmittedProjectionOutcome::Finished);
        }
        self.demand_regions_head(execution, [entity.partition_id])?;
        if let Err(()) = admit_work(super::super::read_prices::KEYED_RECORD) {
            return Ok(AdmittedProjectionOutcome::Stopped);
        }
        let Some(selected) = self.selected_entity_projection(entity) else {
            return Ok(AdmittedProjectionOutcome::Finished(false));
        };
        self.admit_entity_projection_scope(selected.kind.kind_id, scope)?;
        let produced = selected.project(scope, &mut |record| Ok(project(record)))?;
        Ok(AdmittedProjectionOutcome::Finished(produced))
    }

    fn selected_entity_projection(&self, entity: EntityId) -> Option<SelectedEntityProjection<'_>> {
        let root = self.basis.root()?;
        let partition = self.admitted_head_region(entity.partition_id)?;
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
            .expect(/* Proof: The owner selects this slot from the same retained arena after existence and visibility admission; arena materialization installs its kind, extra and creation columns together. */ "visible entity slot has creation metadata");
        Some(SelectedEntityProjection {
            actual,
            slot,
            kind,
            created,
        })
    }
}

enum AdmittedProjectionOutcome {
    Stopped,
    Finished(bool),
}

struct SelectedEntityProjection<'a> {
    actual: EntityId,
    slot: crate::storage::substrate::SlotView<'a, crate::storage::substrate::EntityRecordKind>,
    kind: crate::schema::data::KindResolution,
    created: crate::identity::data::VersionId,
}

impl SelectedEntityProjection<'_> {
    fn project(
        self,
        scope: &ProjectionAspectScope,
        project: &mut dyn for<'a> FnMut(
            EntityProjectionRecord<'a>,
        ) -> Result<bool, RelationalExecutionDenialCause>,
    ) -> Result<bool, RelationalExecutionDenialCause> {
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
