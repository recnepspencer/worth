//! A fixed model/read set with independently growing dictionary vocabulary.
use crate::content::historical_kind::fixture::World;
use std::sync::Arc;
use worth_execution::{
    CancellationSource, ExecutionImmutableBytes, ExecutionRequest, SerialRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    identity::KindId, inspection::observe_symbol_snapshot_materialization,
};
use worth_testkit_relational::maintenance::assert_change_bounded;

fn projection_work(extra_symbols: usize) -> u64 {
    let request = SerialRequest::from_policy(
        &ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(1 << 30),
                Some(100_000_000),
            ),
        ),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let mut world = World::new();
    world.create(&execution, 7, "one selected record");
    let (_, basis) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    for index in 0..extra_symbols {
        let text = format!("unrelated-symbol-{index:06}");
        let bytes =
            ExecutionImmutableBytes::from_external_bytes(Arc::new(text.into_bytes().into()));
        world
            .runtime
            .admit_field_text(&execution, &basis, &bytes)
            .unwrap();
    }
    let snapshot = world
        .runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let pin = world
        .runtime
        .read_truth()
        .pin_read_capability(&snapshot, &[], &execution)
        .unwrap();
    // Vocabulary count is independent of the one-record model/read set.
    assert!(pin
        .symbols()
        .symbol(&format!("unrelated-symbol-{:06}", extra_symbols - 1))
        .is_some());
    let (counts, materialized) = observe_symbol_snapshot_materialization(|| {
        (0..8)
            .map(|_| {
                let projection = pin.project_read();
                let records = projection
                    .bounded_entities_of_kind(&execution, KindId(1), 100)
                    .unwrap()
                    .unwrap();
                records.into_records().len()
            })
            .collect::<Vec<_>>()
    });
    assert!(
        counts.iter().all(|count| *count == 1),
        "the fixed real read must succeed"
    );
    world
        .runtime
        .snapshots()
        .release_snapshot(&snapshot)
        .unwrap();
    materialized
}

#[test]
fn pinned_projection_does_not_materialize_unrelated_symbols() {
    let base = projection_work(512);
    let doubled = projection_work(1024);
    eprintln!("SDK pinned projection snapshot rows: {base}->{doubled}");
    assert_change_bounded(
        "pinned projection symbol materialization",
        "symbols",
        base.max(1),
        doubled.max(1),
    );
}
