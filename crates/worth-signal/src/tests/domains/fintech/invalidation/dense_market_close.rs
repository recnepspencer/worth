use super::super::certification::invalidation::{
    ExpectedLocalityCounterRow, FinancialLocalityExpectationManifest,
    FreshFinancialLocalityRecompute,
};
use super::super::world::{
    compile_financial_locality_world, DensityRatio, FinancialWorldDefinition,
};

#[test]
fn dense_market_close_ready_work_tracks_semantic_density_across_bounded_batches() {
    for (ratio, affected) in [
        (DensityRatio::OneInOneHundred, 10_u64),
        (DensityRatio::OneInFour, 250),
        (DensityRatio::FourInFive, 800),
    ] {
        let definition = FinancialWorldDefinition::dense_market_close(41, 1_000, ratio);
        let mut compiled = compile_financial_locality_world(definition).unwrap();
        let manifest = FinancialLocalityExpectationManifest::derive(
            compiled.locality_definition(),
            compiled.locality_graph_instance(),
        );
        let fresh = FreshFinancialLocalityRecompute::run(compiled.locality_definition());
        let observation = compiled.run_locality_action_trace(0).unwrap();
        let expected = manifest.counter_manifest();

        assert_eq!(
            observation.ready_items_enqueued,
            expected.value(ExpectedLocalityCounterRow::ReadyItemsEnqueued)
        );
        assert_eq!(
            observation.ready_items_popped,
            expected.value(ExpectedLocalityCounterRow::ReadyItemsPopped)
        );
        assert_eq!(observation.ready_items_enqueued, affected);
        assert_eq!(observation.ready_items_popped, affected);
        assert_eq!(observation.retained_ready_width, 0);
        assert_eq!(
            observation.peak_ready_width,
            observation.physical_ready_peak,
        );
        assert!(observation.peak_ready_width > 0);
        assert!(observation.peak_ready_width <= observation.ready_items_enqueued);
        assert_eq!(
            compiled.committed_locality_financial_values().unwrap(),
            *fresh.shocked_values()
        );
    }
}
