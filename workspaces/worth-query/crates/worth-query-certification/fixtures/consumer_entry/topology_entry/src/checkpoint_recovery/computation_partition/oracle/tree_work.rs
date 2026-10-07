//! The public observer reconciles actual reducer entries while partition
//! execution remains incremental, even when the tree's topology changes.

use super::*;
use worth_foundational::facade::PartitionIdentity as Id;
use worth_query_decl::facade::application_operation::application_computation_partition_identity;
use worth_query_host::facade::application_contribution::WorthQueryPartitionedTreeRun as TreeRun;

#[path = "tree_shape.rs"]
mod shape;

pub(super) fn assert_counted(run: &OracleRun) {
    let reported: u128 = run
        .tree_runs
        .iter()
        .map(|(_, tree)| tree.metrics().combine_calls)
        .sum();
    assert_eq!(
        reported,
        u128::try_from(run.combines).unwrap(),
        "every real reducer entry is reported: {:?}",
        run.tree_runs
    );
}

pub(super) fn identity(region: u32) -> Id {
    application_computation_partition_identity(&RegionKey(region), &mut |_| Ok::<(), ()>(()))
        .unwrap()
        .partition()
}

pub(super) fn keys(size: usize) -> Vec<Id> {
    let mut keys: Vec<_> = (0..size)
        .map(|n| identity(u32::try_from(n).unwrap()))
        .collect();
    keys.sort();
    keys
}

/// Root-to-leaf identities for 6.11's immutable node-sharing proof.
pub(super) fn update_path(keys: &[Id], key: Id) -> Vec<Id> {
    shape::Shape::update_path(&shape::Shape::from_sorted(keys), key)
}

pub(super) fn update_bound(keys: &[Id], key: Id) -> u64 {
    u64::try_from(update_path(keys, key).len()).unwrap()
}

pub(super) fn insert_bound(keys: &[Id], key: Id) -> u64 {
    let tree = shape::Shape::from_sorted(keys);
    shape::Shape::depth(&tree, key) + shape::Shape::rotations(&tree, key) + 1
}

pub(super) fn delete_bound(keys: &[Id], key: Id) -> u64 {
    shape::Shape::deletion_depth(&shape::Shape::from_sorted(keys), key)
}

pub(super) fn assert_edited(run: &OracleRun, bound: u64) {
    assert_counted(run);
    let [(WorthQueryPartitionedComputationRun::Incremental, TreeRun::Edited(metrics))] =
        run.tree_runs.as_slice()
    else {
        panic!(
            "partition reuse has its own tree-edit dimension: {:?}",
            run.tree_runs
        )
    };
    assert!(
        metrics.recombined_nodes <= u128::from(bound),
        "recombined {} nodes, pre-edit shape bound {bound}",
        metrics.recombined_nodes
    );
    assert_eq!(metrics.combine_calls, 2 * metrics.recombined_nodes);
}

#[test]
fn recomputed_partition_with_identical_bits_reports_zero_tree_work() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(|graph| {
        facts::seed_set(graph, "even", -0.0);
        for place in 0..32 {
            seed_entry(
                graph,
                &["even"],
                place,
                RegionEntry {
                    id: u64::try_from(place).unwrap(),
                    region: u32::try_from(place).unwrap(),
                    value: 1.0,
                    work: 1,
                    fault: None,
                },
            );
        }
    });
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (_, first) = demand(&request, &application);
    assert_counted(&first[0]);
    // An ignored probe fault moves a gathered fact while preserving the
    // leaf's canonical bits. Owner calls are independent of tree work.
    edit(
        &request,
        &application,
        EntryEdit::new("even", 7, super::super::entry_edit::EntryFact::Fault, 3),
        1,
    );
    let (_, next) = demand(&request, &application);
    assert_eq!(next[0].calls.kernels, 1);
    assert_eq!(
        next[0].runs,
        [WorthQueryPartitionedComputationRun::Incremental]
    );
    assert_edited(&next[0], 0);
    assert_eq!(
        next[0].outcome.as_ref().unwrap().0,
        first[0].outcome.as_ref().unwrap().0
    );
}
