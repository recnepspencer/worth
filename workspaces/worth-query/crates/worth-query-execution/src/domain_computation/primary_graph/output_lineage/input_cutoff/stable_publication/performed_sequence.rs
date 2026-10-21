//! Remove only the selected alias's checked witness span for republication.
use super::StableFactSpans;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl;
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;

pub(in crate::domain_computation::primary_graph::output_lineage) fn performed_fact_sequence(
    alias_facts: &FactPayload,
    spans: &StableFactSpans,
    prefix_count: usize,

    control: StorageControl<'_, '_>,
) -> Option<FactPayload> {
    let facts = spans
        .performed_sequence(alias_facts, prefix_count, control)
        .ok()?;
    Some(facts)
}
