//! The public observer reconciles actual reducer entries while partition
//! execution remains incremental, even when the tree's topology changes.

use super::*;
use worth_foundational::facade::PartitionIdentity as Id;
use worth_query_decl::facade::application_operation::application_computation_partition_identity;
use worth_query_host::facade::application_contribution::WorthQueryPartitionedTreeRun as TreeRun;

#[path = "tree_shape.rs"]
mod shape;

pub(super) enum TreeStop<'a> {
    Exact,
    BeforeTree,
    ParallelBuild { serial: &'a [TreeRun] },
}

pub(super) fn stop_kind<'a>(run: &OracleRun, serial: Option<&'a [TreeRun]>) -> TreeStop<'a> {
    use worth_query_host::facade::primary_graph::WorthQueryExecutionPlacementForTest as Placement;
    if run.outcome.is_err() && run.tree_runs.is_empty() {
        TreeStop::BeforeTree
    } else if run.outcome.is_err()
        && matches!(run.tree_runs.as_slice(), [TreeRun::Full(_, _)])
        && matches!(run.placement, Placement::Leased(n) | Placement::Certified { workers: n, .. } if n.get() > 1)
    {
        TreeStop::ParallelBuild {
            serial: serial.expect("a stopped parallel build requires its serial witness"),
        }
    } else {
        TreeStop::Exact
    }
}

pub(super) fn assert_counted(run: &OracleRun) {
    assert_reconciled(run, TreeStop::Exact);
}

pub(super) fn assert_reconciled(run: &OracleRun, stop: TreeStop<'_>) {
    use worth_query_host::facade::primary_graph::WorthQueryExecutionPlacementForTest as Placement;
    let full_completed = run
        .runs
        .iter()
        .any(|run| matches!(run, WorthQueryPartitionedComputationRun::Full(_)));
    // Two diagnostic certification builds follow the reported boundary.
    // The owner independently gathered P leaves; each diagnostic build
    // completes P valid f64 pairs. Exclude those later calls from this run.
    let certified_leaves = if full_completed && matches!(run.placement, Placement::Certified { .. })
    {
        assert_eq!(run.calls.kernels, 3 * run.calls.gathers);
        u128::try_from(run.calls.gathers).unwrap()
    } else {
        0
    };
    let reported_calls: u128 = run
        .tree_runs
        .iter()
        .map(|tree| tree.metrics().combine_calls)
        .sum();
    let reported_nodes: u128 = run
        .tree_runs
        .iter()
        .map(|tree| tree.metrics().recombined_nodes)
        .sum();
    let counted_calls = u128::try_from(run.combines)
        .unwrap()
        .checked_sub(4 * certified_leaves)
        .unwrap();
    let counted_nodes = u128::try_from(run.tree_nodes)
        .unwrap()
        .checked_sub(2 * certified_leaves)
        .unwrap();
    if let TreeStop::ParallelBuild { serial } = stop {
        assert_eq!(
            run.tree_runs, serial,
            "stopped parallel work equals the serial prefix"
        );
        assert!(
            reported_calls <= counted_calls,
            "canonical calls exceed independently entered calls: {run:?}"
        );
        assert!(
            reported_nodes <= counted_nodes,
            "canonical nodes exceed independently completed pairs: {run:?}"
        );
    } else {
        if matches!(stop, TreeStop::BeforeTree) {
            assert_eq!(
                (reported_calls, reported_nodes, counted_calls, counted_nodes),
                (0, 0, 0, 0),
                "map-stage stops do no tree work"
            );
        }
        assert_eq!(
            reported_calls, counted_calls,
            "every reducer entry reconciles: {run:?}"
        );
        assert_eq!(
            reported_nodes, counted_nodes,
            "every completed combine pair reconciles: {run:?}"
        );
    }
}

pub(super) fn priority(key: Id) -> (u64, Id) {
    shape::priority(key)
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
    let [TreeRun::Edited(metrics)] = run.tree_runs.as_slice() else {
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
