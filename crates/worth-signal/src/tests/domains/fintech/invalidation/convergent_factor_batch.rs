use super::super::certification::invalidation::{
    ExpectedLocalityCounterRow, FinancialLocalityExpectationManifest,
    FreshFinancialLocalityRecompute,
};
use super::super::world::{
    compile_financial_locality_world, FinancialLocalityRedObservation, FinancialWorldDefinition,
};

#[test]
fn convergent_factor_batch_preserves_four_causes_across_all_seed_orders() {
    let mut canonical_counters = None;
    let mut canonical_values = None;
    for trace_index in 0..24 {
        let definition = FinancialWorldDefinition::convergent_factor_batch(41, 0);
        let mut compiled = compile_financial_locality_world(definition).unwrap();
        let manifest = FinancialLocalityExpectationManifest::derive(
            compiled.locality_definition(),
            compiled.locality_graph_instance(),
        );
        let fresh = FreshFinancialLocalityRecompute::run(compiled.locality_definition());
        let observation = compiled.run_locality_action_trace(trace_index).unwrap();
        assert_scheduling_counters(&observation, &manifest);
        let values = compiled.committed_locality_financial_values().unwrap();
        assert_eq!(values, *fresh.shocked_values());
        let counters = scheduling_tuple(&observation);
        assert_eq!(*canonical_counters.get_or_insert(counters), counters);
        assert_eq!(*canonical_values.get_or_insert(values.clone()), values);
    }
}

#[test]
fn convergent_factor_batch_repeated_admission_merges_without_an_extra_queue_item() {
    let definition = FinancialWorldDefinition::convergent_factor_batch(41, 1);
    let mut compiled = compile_financial_locality_world(definition).unwrap();
    let manifest = FinancialLocalityExpectationManifest::derive(
        compiled.locality_definition(),
        compiled.locality_graph_instance(),
    );
    let observation = compiled.run_locality_action_trace(0).unwrap();

    assert_scheduling_counters(&observation, &manifest);
    assert_eq!(observation.work_items_merged, 1);
    assert_eq!(observation.ready_items_enqueued, 5);
    assert_eq!(observation.ready_items_popped, 5);
}

fn assert_scheduling_counters(
    observation: &FinancialLocalityRedObservation,
    manifest: &FinancialLocalityExpectationManifest,
) {
    let expected = manifest.counter_manifest();
    assert_eq!(
        observation.work_items_admitted,
        expected.value(ExpectedLocalityCounterRow::WorkItemsAdmitted)
    );
    assert_eq!(
        observation.work_items_merged,
        expected.value(ExpectedLocalityCounterRow::WorkItemsMerged)
    );
    assert_eq!(
        observation.ready_items_enqueued,
        expected.value(ExpectedLocalityCounterRow::ReadyItemsEnqueued)
    );
    assert_eq!(
        observation.ready_items_popped,
        expected.value(ExpectedLocalityCounterRow::ReadyItemsPopped)
    );
    assert_eq!(
        observation.retained_ready_width,
        expected.value(ExpectedLocalityCounterRow::RetainedReadyFrontierWidth)
    );
    assert_eq!(
        observation.peak_ready_width,
        observation.physical_ready_peak,
    );
}

fn scheduling_tuple(observation: &FinancialLocalityRedObservation) -> (u64, u64, u64, u64) {
    (
        observation.work_items_admitted,
        observation.work_items_merged,
        observation.ready_items_enqueued,
        observation.ready_items_popped,
    )
}
