//! Replacing retained facts discards their exact alias-local layout.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::StorageControl,
    application_checkpoint::facts::NativeFactPayloadBuilder,
};

#[test]
fn replacing_retained_body_invalidates_its_alias_layout_even_for_equal_data() {
    let control = StorageControl::new(
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        None,
    );
    let source = super::super::ComputationSourceEvidence::for_test(false);
    let body = native(8, control);
    let spans =
        super::super::input_cutoff::stable_publication::StableFactSpans::new(body.clone(), 3, 2)
            .unwrap();
    let mut row = RecordedOutputMutable::new(None, Some(source.retain_facts(body.clone())), None,
        super::super::retained_computation::RecordedComputation::Absent(crate::domain_computation::primary_graph::application_contribution::PriorAbsence::FirstRun)).with_stable_fact_spans(spans.clone());
    assert!(row.stable_fact_spans.as_ref().unwrap().matches(&body));
    row.replace_retained_facts(source.retain_facts(native(8, control)));
    assert!(row.stable_fact_spans.is_none());
}

fn native(
    count: usize,
    control: StorageControl<'_, '_>,
) -> crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload {
    let mut author = NativeFactPayloadBuilder::allocate(count, control).unwrap();
    for ordinal in 0..count {
        author.push(crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact::SourceEntity { entity_id: worth_relational::facade::identity::EntityId::new(worth_relational::facade::identity::PartitionId::main(), ordinal as u64, 1) }).unwrap();
    }
    author.seal().unwrap()
}
