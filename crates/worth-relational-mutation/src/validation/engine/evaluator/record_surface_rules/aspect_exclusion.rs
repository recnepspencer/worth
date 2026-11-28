//! Evaluate the installed group roster against the applied, touched post-state.
use crate::contracts::diagnostics::data::artifacts::DiagnosticCode;
use crate::contracts::validation::data::execution::InvariantClass;
use crate::contracts::validation::data::results::{InvariantViolation, InvariantViolationFields};

use super::super::super::context::InvariantExecutionContext;

// Work is proportional to touched slots plus group-member lookups on live
// touched entities. Neither untouched records nor intermediate edits are read.
pub(in super::super) fn evaluate(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
) -> Option<InvariantViolation> {
    let view = context.enforcement_state_view();
    let touched = view.touched_visible_entity_ids_with_budget(|slots| {
        context.checkpoint(slots as u64)
            && context.claim_scratch((slots as u64).saturating_mul(size_of::<
                crate::contracts::identity::data::record_ids::EntityId,
            >() as u64))
    })?;
    for &entity_id in touched.iter() {
        if !context.checkpoint(1) {
            return None;
        }
        let Some(kind_id) = view
            .entity_metadata(entity_id)
            .map(|metadata| metadata.kind_id)
        else {
            continue;
        };
        let Some(plan) = context.entity_aspect_plan(kind_id) else {
            continue;
        };
        if plan.exclusion_groups.is_empty() {
            continue;
        }
        let state = view.entity_aspect_state(entity_id);
        for group in &plan.exclusion_groups {
            if !context.checkpoint(group.aspects.len() as u64) {
                return None;
            }
            let active_count = group
                .aspects
                .iter()
                .filter(|aspect| state.is_some_and(|state| state.get(aspect).is_some()))
                .count();
            if active_count > 1 || (group.required && active_count == 0) {
                if !context.claim_result(
                    4096_u64.saturating_add((group.group_id.len() as u64).saturating_mul(8)),
                ) {
                    return None;
                }
                return Some(InvariantViolation {
                    class,
                    code: DiagnosticCode::InvariantViolation,
                    detail: format!("entity {entity_id:?} has {active_count} active aspects in exclusion group {:?}", group.group_id),
                    fields: InvariantViolationFields::EntityAspectExclusion {
                        entity_id,
                        kind_id,
                        group_id: group.group_id.clone(),
                        required: group.required,
                        active_count,
                    },
                });
            }
        }
    }
    None
}
