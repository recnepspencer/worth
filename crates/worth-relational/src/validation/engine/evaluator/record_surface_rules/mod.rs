mod live_record_sidecars;
mod snapshot_entity_limits;

use crate::validation::data::{InvariantClass, InvariantViolation, RecordKindTag};

use super::super::context::InvariantExecutionContext;

fn single_violation(violation: Option<InvariantViolation>) -> Vec<InvariantViolation> {
    violation.into_iter().collect()
}

pub(super) fn evaluate_record_surface_rule(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    kind: &RecordKindTag,
) -> Vec<InvariantViolation> {
    single_violation(live_record_sidecars::evaluate_live_record_sidecar_rule(
        context, class, kind,
    ))
}

pub(super) fn evaluate_snapshot_entity_limit_rule(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    limit: usize,
) -> Option<InvariantViolation> {
    snapshot_entity_limits::evaluate_max_snapshot_entities(context, class, limit)
}
