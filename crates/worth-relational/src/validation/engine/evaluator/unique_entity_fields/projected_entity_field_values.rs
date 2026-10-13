use super::super::super::context::InvariantExecutionContext;
use crate::validation::engine::state_view::VisibleEntityMetadata;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};

#[derive(Debug, Clone)]
pub(super) struct ProjectedEntityAspectFieldValue {
    pub(super) value: AspectValue,
}

pub(super) fn projected_entity_aspect_field_value_for_metadata(
    context: &InvariantExecutionContext<'_, '_>,
    metadata: &VisibleEntityMetadata,
    field_locator: &AspectFieldLocator,
) -> Option<ProjectedEntityAspectFieldValue> {
    let state_view = context.enforcement_state_view();
    let value =
        context.projected_entity_aspect_field_value(metadata.kind_id, field_locator, || {
            state_view.entity_aspect_state(metadata.entity_id)
        })?;
    // Both the installed projection proof and record are borrowed. Admission
    // covers only the selected field clone and comparison-key retention.
    let value_bytes = value.owned_allocation_capacity_bytes() as u64;
    if !context.check_scratch_peak(value_bytes.saturating_mul(4).saturating_add(128)) {
        return None;
    }
    Some(ProjectedEntityAspectFieldValue {
        value: value.clone(),
    })
}
