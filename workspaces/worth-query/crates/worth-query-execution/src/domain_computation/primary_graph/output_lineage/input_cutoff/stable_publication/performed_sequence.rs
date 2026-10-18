//! Remove only the selected alias's checked witness span for republication.
use super::StableFactSpans;
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::StorageControl,
    application_checkpoint::facts::FactPayload,
    output_lineage::{
        invalidation::InvalidationEditAdmission,
        retained_capacity::{LineageRetentionLedger, RetainedLineageCapacity},
    },
};

pub(in crate::domain_computation::primary_graph::output_lineage) fn performed_fact_sequence(
    alias_facts: &FactPayload,
    spans: &StableFactSpans,
    prefix_count: usize,
    retention: &LineageRetentionLedger,
    admission: &mut InvalidationEditAdmission,
    control: StorageControl<'_, '_>,
) -> Option<(FactPayload, RetainedLineageCapacity)> {
    admission.charge_external_work(3).ok()?;
    let facts = spans
        .performed_sequence(alias_facts, prefix_count, control)
        .ok()?;
    // Retention policy counts complete held backing; this cached descriptor
    // neither copies the prefix nor reserves a second native heap allocation.
    let capacity = retention.reserve(facts.retained_resident_bytes()?).ok()?;
    Some((facts, capacity))
}
