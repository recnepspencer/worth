//! A true installed input cutoff, with no fabricated Native execution evidence.
use super::*;
use installation::ReuseSchema;
use primary_graph::application_output_demand::OriginalSettledProducerSource;
use std::sync::Arc;

#[test]
fn genuine_input_cutoff_retains_stable_supplier_without_native_execution() {
    let runtime = installation::install::<ReuseSchema>(false);
    let initial = journey::demand(&runtime, true).unwrap();
    court::edit_source(&runtime);
    let performed = journey::demand(&runtime, true).unwrap();
    assert_eq!(
        performed.producer_identity(),
        <Preserve<ReuseSchema> as WorthQueryApplicationProducerBinding<ReuseSchema>>::IDENTITY
    );
    // A changed projected value advances the genuine source meaning, while
    // the current canonical KeepInput and actual handler dependencies agree.
    court::edit_source_to(&runtime, 3);
    let stable = journey::demand_with_contacts(
        &runtime,
        true,
        0,
        WorthQueryOutputSettlementPosture::StableReused,
    )
    .unwrap();
    assert!(stable.application_commit_receipt().is_none());
    assert!(stable
        .readiness_delivery()
        .unwrap()
        .causal_observation()
        .is_none());
    runtime
        .with_host_advancement(|phase| {
            let read = stable.retained_read();
            let selected = runtime
                .select_application_read_observation(&phase, &read)
                .unwrap();
            let seed = selected.original_settled_producer().unwrap();
            let OriginalSettledProducerSource::StableReused { settlement } = seed.source() else {
                panic!("a genuine input cutoff retains only its stable lineage supplier")
            };
            assert!(Arc::ptr_eq(
                settlement,
                stable.stable.as_ref().unwrap().exact_settlement()
            ));
            assert_eq!(seed.read().selected_commit(), read.selected_commit());
            assert!(seed
                .correspondence()
                .native_witness_roles()
                .eq(stable.output_correspondence.native_witness_roles()));
        })
        .unwrap();
    drop((initial, performed));
}
