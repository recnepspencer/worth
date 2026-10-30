#[allow(unused_imports)]
use crate::decompose_pattern::ordinary_context::tree_reuse::*;
pub(crate) use {
    crate::decompose_pattern::ordinary_context::EDITIONS, worth_execution::BackInput,
    worth_execution::ExecutionDecompose, worth_execution::ExecutionMap,
    worth_execution::InterfaceSolution, worth_execution::InteriorResult,
    worth_execution::MapKernelFailure, worth_execution::MapPartition,
    worth_foundational::PartitionIdentity,
};

#[test]
fn one_changed_contribution_recombines_only_path_and_matches_fresh_tree() {
    let original: Vec<_> = (1..=64).map(|id| (id, id)).collect();
    let mut retained = decomposer();
    let first = retained
        .run(None, &map(&original), EDITIONS, interior, interface, back)
        .unwrap();
    assert_eq!(recombined(&first.reduction_metrics), 64);

    let updated = retained
        .run(
            None,
            &map(&[(31, 100)]),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    assert!(recombined(&updated.reduction_metrics) > 0);
    assert!(recombined(&updated.reduction_metrics) < 64);
    assert_eq!(updated.reuse.unchanged_contributions, 63);

    let mut fresh_entries = original;
    fresh_entries[30].1 = 100;
    let fresh = decomposer()
        .run(
            None,
            &map(&fresh_entries),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    assert_eq!(updated.values, fresh.values);
}
