use worth_foundational::facade::AspectFieldLocator;

use crate::identity::data::{EntityId, RelationId};
use crate::storage::data::RelationalFieldRevision;

use super::VisibilityProjectionView;

#[path = "field_revisions/admitted_entity.rs"]
mod admitted_entity;
#[path = "field_revisions/relation_revision.rs"]
mod relation_revision;

impl VisibilityProjectionView {
    /// Native field status and revision at an exact immutable root. `None`
    /// means unavailable provenance; it never means an absent field. Current
    /// checkpoint images retain native revisions, including absent revisions.
    pub fn entity_field_revision(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
        locator: &AspectFieldLocator,
    ) -> Result<Option<RelationalFieldRevision>, crate::execution::RelationalExecutionDenialCause>
    {
        super::super::read_work::metered(
            execution,
            |context| {
                self.entity_field_revision_admitted(execution, entity, locator, |units| {
                    context.checkpoint(units)
                })
            },
            |_| 0,
        )??
        .map_err(crate::execution::denial::kernel_stop)
    }

    pub fn relation_field_revision(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relation: RelationId,
        locator: &AspectFieldLocator,
    ) -> Result<Option<RelationalFieldRevision>, crate::execution::RelationalExecutionDenialCause>
    {
        crate::execution::check_request(execution)?;
        if !self.is_exact_basis() {
            return Ok(None);
        }
        self.demand_regions_head(execution, [relation.partition_id])?;
        let Some(partition) = self
            .selected_root()
            .and_then(|root| root.admitted_head_region(relation.partition_id))
        else {
            return Ok(None);
        };
        super::super::read_work::metered(
            execution,
            |context| {
                self.read_relation_field_revision_funded(partition, relation, locator, |units| {
                    context.checkpoint(units)
                })
            },
            |_| 0,
        )?
        .map_err(crate::execution::denial::kernel_stop)
    }
}

#[cfg(test)]
#[path = "field_revisions/restore_tests.rs"]
mod restore_tests;

#[cfg(test)]
#[path = "field_revisions/tests.rs"]
mod tests;
