//! Real tree-update work, counted independently at reducer entry, including
//! successful edits followed by a failed attempt and a complete rebuild.

use std::cell::Cell;
use std::collections::BTreeMap;

use worth_execution::{ReductionPlan, ReductionTree};
use worth_foundational::facade::PartitionIdentity as Id;

use super::super::retained::RetainedPartitions;
use super::super::tree_report::{
    WorthQueryPartitionedTreeRebuildCause as Rebuild, WorthQueryPartitionedTreeRun as TreeRun,
};
use super::super::tree_update::next_tree;
use super::*;

#[path = "tree_attempts.rs"]
mod attempts;
#[path = "tree_mean.rs"]
mod mean;
#[path = "tree_shape.rs"]
mod shape;

thread_local! {
    static CALLS: Cell<u64> = const { Cell::new(0) };
    static PANIC_AT: Cell<Option<u64>> = const { Cell::new(None) };
}

fn counted(left: &u64, right: &u64) -> u64 {
    let entered = CALLS.get() + 1;
    CALLS.set(entered);
    if PANIC_AT.get() == Some(entered) {
        PANIC_AT.set(None);
        panic!("one refused combine attempt");
    }
    left + right
}

type Retained = RetainedPartitions<
    Parity,
    <Owner as WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>>::Item,
    u64,
>;
type Tree = ReductionTree<u64, fn(&u64, &u64) -> u64>;

fn template() -> SealedComputationRun {
    let world = installed_authorization_world(true);
    // Keep the source run's memory reservation alive throughout each test.
    first_run(&world, &installed(StatusRead::Gather(1), sum))
        .sealed
        .unwrap()
        .unwrap()
}

/// Only the tree-update boundary is under test here. Unused owner facts are
/// retained from a real admitted run; local identities carry no authority.
fn retained(source: &SealedComputationRun, keys: &[Id]) -> Retained {
    let template = source.state.typed.downcast_ref::<Retained>().unwrap();
    let (tree, _) = Tree::try_from_declared(
        plan(keys),
        keys.iter().map(|id| (*id, 1)).collect(),
        0,
        counted as fn(&u64, &u64) -> u64,
    )
    .unwrap();
    let partition = template.partitions.values().next().unwrap();
    Retained {
        items: Arc::clone(&template.items),
        digests: template.digests.clone(),
        membership: template.membership.clone(),
        item_keys: Arc::clone(&template.item_keys),
        routing: Arc::clone(&template.routing),
        partitions: keys.iter().map(|id| (*id, Arc::clone(partition))).collect(),
        tree,
    }
}

fn plan(keys: &[Id]) -> ReductionPlan {
    ReductionPlan::try_from_sorted_unique(keys.to_vec()).unwrap()
}

// Use the execution test lane's governed serial policy. Its 64 GiB room
// admits the declared path-copy holds of the fixed mean-law workload.
fn serial_placement() -> RuntimeWorldExecutionPlacement<'static> {
    use super::super::super::super::request_execution::test_policy;
    RuntimeWorldExecutionPlacement::Serial(test_policy(std::num::NonZeroUsize::MIN, 1 << 36))
}

fn run(
    retained: &Retained,
    keys: &[Id],
    results: BTreeMap<Id, u64>,
    remaining: u64,
) -> super::super::tree_report::ReportedTree<Tree, u32> {
    let request = live_scope();
    let execution = QueryRequestExecution::open(serial_placement(), &request);
    let results_bytes = u64::try_from(results.len() * size_of::<(Id, u64)>()).unwrap();
    let mut memory = execution.reserve(results_bytes).unwrap();
    let declared = 4096;
    let reducer = WorthQueryDeterministicReducer::canonical(|| 0, counted);
    let plan = plan(keys);
    let work = plan.checked_build_work();
    next_tree(
        retained,
        plan,
        work,
        results,
        remaining,
        declared,
        &reducer,
        &execution,
        &mut memory,
    )
}

fn reconcile(report: TreeRun) {
    assert_eq!(
        report.metrics().combine_calls,
        u128::from(CALLS.get()),
        "reported combine entries equal independent reducer entries"
    );
}

#[test]
fn every_edit_obeys_its_pre_edit_shape_bound_and_reconciles() {
    let template = template();
    let keys: Vec<_> = (0..128).map(|n| Id::new(n * 17 + 3)).collect();
    let retained = retained(&template, &keys);
    let shape = shape::Shape::from_sorted(&keys);
    for key in &keys {
        CALLS.set(0);
        let next = run(&retained, &keys, BTreeMap::from([(*key, 2)]), u64::MAX);
        reconcile(next.report);
        let TreeRun::Edited(metrics) = next.report else {
            panic!("an ordinary update edits")
        };
        let path = shape::Shape::update_path(&shape, *key);
        assert!(metrics.recombined_nodes <= u128::try_from(path.len()).unwrap());
        assert_eq!(metrics.combine_calls, 2 * metrics.recombined_nodes);
        assert_eq!(*next.outcome.unwrap().result(), 129);
        CALLS.set(0);
        let same = run(&retained, &keys, BTreeMap::from([(*key, 1)]), u64::MAX);
        reconcile(same.report);
        assert_eq!(
            same.report.metrics().recombined_nodes,
            0,
            "identical bits cut off"
        );
        assert_eq!(same.report.metrics().combine_calls, 0);
        let deleted: Vec<_> = keys.iter().copied().filter(|id| id != key).collect();
        CALLS.set(0);
        let next = run(&retained, &deleted, BTreeMap::new(), u64::MAX);
        reconcile(next.report);
        assert!(matches!(next.report, TreeRun::Edited(_)));
        assert_eq!(
            next.report.metrics().combine_calls,
            2 * next.report.metrics().recombined_nodes
        );
        assert!(
            next.report.metrics().recombined_nodes
                <= u128::from(shape::Shape::deletion_depth(&shape, *key))
        );
        assert_eq!(*next.outcome.unwrap().result(), 127);
    }
    for key in (0..128).map(|n| Id::new(n * 17 + 4)) {
        let mut inserted = keys.clone();
        inserted.push(key);
        inserted.sort();
        let bound = shape::Shape::depth(&shape, key) + shape::Shape::rotations(&shape, key) + 1;
        CALLS.set(0);
        let next = run(&retained, &inserted, BTreeMap::from([(key, 2)]), u64::MAX);
        reconcile(next.report);
        assert!(matches!(next.report, TreeRun::Edited(_)));
        assert_eq!(
            next.report.metrics().combine_calls,
            2 * next.report.metrics().recombined_nodes
        );
        assert!(next.report.metrics().recombined_nodes <= u128::from(bound));
        assert_eq!(*next.outcome.unwrap().result(), 130);
    }
}

#[test]
fn public_incremental_observation_reports_actual_work_apart_from_charge() {
    let world = installed_authorization_world(true);
    let installed = installed(StatusRead::Gather(1), counted);
    let first = first_run(&world, &installed);
    let charge = first.outcome.as_ref().unwrap().1;
    *installed.owner.bump.lock().unwrap() = 10;
    CALLS.set(0);
    let next = attempt(&world, &installed, Some(prior_of(first, true)));
    assert_eq!(next.runs, [(Run::Incremental, None)]);
    let [(partitions, tree)] = next.tree_runs.as_slice() else {
        panic!("one terminal report")
    };
    assert_eq!(*partitions, Run::Incremental);
    reconcile(*tree);
    assert!(matches!(tree, TreeRun::Edited(_)));
    assert_eq!(next.outcome.unwrap().1, charge);
}
