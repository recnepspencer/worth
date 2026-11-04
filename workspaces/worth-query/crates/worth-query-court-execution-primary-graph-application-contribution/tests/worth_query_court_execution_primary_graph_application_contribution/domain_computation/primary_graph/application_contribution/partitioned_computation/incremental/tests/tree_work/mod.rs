//! Real tree-update work, counted independently at reducer entry, including
//! successful edits followed by a failed attempt and a complete rebuild.

pub use std::cell::Cell;
pub use std::collections::BTreeMap;

pub use worth_execution::NativeReductionTree;
pub use worth_execution::ReductionPlan;
pub use worth_execution::ReductionTree;
pub use worth_foundational::facade::PartitionIdentity as Id;

pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::retained::RetainedPartitions;
pub use worth_query_execution::facade::application_contribution::WorthQueryPartitionedTreeRebuildCause as Rebuild;
pub use worth_query_execution::facade::application_contribution::WorthQueryPartitionedTreeRun as TreeRun;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tree_update::next_tree;
pub use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::*;

thread_local! {
    pub static CALLS: Cell<u64> = const { Cell::new(0) };
    pub static NODES: Cell<u64> = const { Cell::new(0) };
    pub static PAIR: Cell<bool> = const { Cell::new(false) };
    pub static PANIC_AT: Cell<Option<u64>> = const { Cell::new(None) };
}

pub fn counted(left: &u64, right: &u64) -> u64 {
    let entered = CALLS.get() + 1;
    CALLS.set(entered);
    if PANIC_AT.get() == Some(entered) {
        PANIC_AT.set(None);
        PAIR.set(false);
        panic!("one refused combine attempt");
    }
    let value = left + right;
    if PAIR.replace(!PAIR.get()) {
        NODES.set(NODES.get() + 1);
    }
    value
}

pub type Retained = RetainedPartitions<
    Parity,
    <Owner as WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>>::Item,
    u64,
>;
pub type Tree = NativeReductionTree<u64, fn(&u64, &u64) -> u64>;

pub fn template() -> SealedComputationRun {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    // Keep the source run's memory reservation alive throughout each test.
    first_run(&world, &installed(StatusRead::Gather(1), sum))
        .sealed
        .unwrap()
        .unwrap()
}

/// Only the tree-update boundary is under test here. Unused owner facts are
/// retained from a real admitted run; local identities carry no authority.
pub fn retained(source: &SealedComputationRun, keys: &[Id]) -> Retained {
    let template = (*(*source.state_for_test()).typed_for_test())
        .downcast_ref::<Retained>()
        .unwrap();
    let request = live_scope();
    let execution = QueryRequestExecution::open_for_test(serial_placement(), &request);
    let (tree, _) = ReductionTree::<u64, fn(&u64, &u64) -> u64>::try_from_declared_native(
        plan(keys),
        keys.iter().map(|id| (*id, 1)).collect(),
        0,
        counted as fn(&u64, &u64) -> u64,
        Some(
            execution
                .dispatch_for_test()
                .unwrap()
                .native_lease_for_test(),
        ),
        || execution.checkpoint_for_test(),
    )
    .unwrap();
    let partition = (*template.partitions_for_test())
        .values_for_test()
        .next()
        .unwrap();
    let policy = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::test_placement::test_policy(std::num::NonZeroUsize::MIN, Some(1 << 36));
    let native = worth_execution::SerialRequest::from_policy(
        &policy,
        worth_execution::CancellationToken::new(),
        None,
    );
    let map = worth_execution::ExecutionMap::from_native_partitions(
        keys.iter().map(|id| (*id, ())).collect(),
    )
    .unwrap();
    let worth_execution::RetainedRunOutcome::Complete {
        run: native_run, ..
    } = map
        .run_retained_run(
            worth_execution::ExecutionRequest::serial(&native),
            worth_execution::ExecutionWorkCeiling::new(None),
            |_, context| {
                context.checkpoint(1)?;
                Ok::<_, worth_execution::MapKernelFailure<()>>(1_u64)
            },
        )
        .unwrap()
    else {
        panic!("actual native scalar slots")
    };
    let partitions = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::partition_metadata::PartitionMetadata::from_recorded_for_test(
        keys.iter()
            .map(|id| (*id, (Arc::clone(&(*partition.key_for_test())), (*partition.gather_for_test()).clone())))
            .collect(),
        execution.allocation_policy_for_test().unwrap(),
    )
    .unwrap();
    Retained::new_for_test(
        Arc::clone(&(*template.items_for_test())),
        (*template.digests_for_test()).clone(),
        (*template.membership_for_test()).clone(),
        Arc::clone(&(*template.item_keys_for_test())),
        Arc::clone(&(*template.call_sequence_for_test())),
        Arc::clone(&(*template.routing_for_test())),
        partitions,
        native_run,
        tree,
    )
}

pub fn plan(keys: &[Id]) -> ReductionPlan {
    ReductionPlan::try_from_sorted_unique(keys.to_vec()).unwrap()
}

// Use the execution test lane's serial policy without a payload forecast.
pub fn serial_placement() -> RuntimeWorldExecutionPlacement<'static> {
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::test_placement::test_policy;
    RuntimeWorldExecutionPlacement::Serial(test_policy(std::num::NonZeroUsize::MIN, None))
}

pub fn run(
    retained: &Retained,
    keys: &[Id],
    results: BTreeMap<Id, u64>,
    remaining: u64,
) -> worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tree_report::ReportedTree<Tree, u32>{
    let request = live_scope();
    let execution = QueryRequestExecution::open_for_test(serial_placement(), &request);
    let reducer = WorthQueryDeterministicReducer::canonical(|| 0, counted);
    let plan = plan(keys);
    let work = plan.checked_build_work();
    next_tree(
        retained,
        plan,
        work,
        results,
        Some(remaining),
        &reducer,
        &execution,
    )
}

pub fn reconcile(report: TreeRun) {
    assert_eq!(
        report
            .metrics_for_test()
            .accounted_metrics()
            .expect("explicit accounted fixture")
            .recombined_nodes,
        u128::from(NODES.get()),
        "independently completed combine pairs"
    );
    assert_eq!(
        report
            .metrics_for_test()
            .accounted_metrics()
            .expect("explicit accounted fixture")
            .combine_calls,
        u128::from(CALLS.get()),
        "reported combine entries equal independent reducer entries"
    );
}

#[test]
fn every_edit_obeys_its_pre_edit_shape_bound_and_reconciles() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let template = template();
    let keys: Vec<_> = (0..128).map(|n| Id::new(n * 17 + 3)).collect();
    let retained = retained(&template, &keys);
    let shape = crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::tree_work::shape::Shape::from_sorted(&keys);
    for key in &keys {
        reset_counts();
        let next = run(&retained, &keys, BTreeMap::from([(*key, 2)]), u64::MAX);
        reconcile((*next.report_for_test()));
        let TreeRun::Edited(metrics) = (*next.report_for_test()) else {
            panic!("an ordinary update edits")
        };
        let metrics = metrics
            .accounted_metrics()
            .expect("explicit accounted fixture");
        let path = crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::tree_work::shape::Shape::update_path(&shape, *key);
        assert!(metrics.recombined_nodes <= u128::try_from(path.len()).unwrap());
        assert_eq!(metrics.combine_calls, 2 * metrics.recombined_nodes);
        assert_eq!(*next.outcome_into_for_test().unwrap().result(), 129);
        reset_counts();
        let same = run(&retained, &keys, BTreeMap::from([(*key, 1)]), u64::MAX);
        reconcile((*same.report_for_test()));
        assert_eq!(
            (*same.report_for_test())
                .metrics_for_test()
                .accounted_metrics()
                .expect("explicit accounted fixture")
                .recombined_nodes,
            0,
            "identical bits cut off"
        );
        assert_eq!(
            (*same.report_for_test())
                .metrics_for_test()
                .accounted_metrics()
                .expect("explicit accounted fixture")
                .combine_calls,
            0
        );
        let deleted: Vec<_> = keys.iter().copied().filter(|id| id != key).collect();
        reset_counts();
        let next = run(&retained, &deleted, BTreeMap::new(), u64::MAX);
        reconcile((*next.report_for_test()));
        assert!(matches!((*next.report_for_test()), TreeRun::Edited(_)));
        assert_eq!(
            (*next.report_for_test())
                .metrics_for_test()
                .accounted_metrics()
                .expect("explicit accounted fixture")
                .combine_calls,
            2 * (*next.report_for_test())
                .metrics_for_test()
                .accounted_metrics()
                .expect("explicit accounted fixture")
                .recombined_nodes
        );
        assert!(
            (*next.report_for_test())
                .metrics_for_test()
                .accounted_metrics()
                .expect("explicit accounted fixture")
                .recombined_nodes
                <= u128::from(crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::tree_work::shape::Shape::deletion_depth(&shape, *key))
        );
        assert_eq!(*next.outcome_into_for_test().unwrap().result(), 127);
    }
    for key in (0..128).map(|n| Id::new(n * 17 + 4)) {
        let mut inserted = keys.clone();
        inserted.push(key);
        inserted.sort();
        let bound = crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::tree_work::shape::Shape::depth(&shape, key) + crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::tree_work::shape::Shape::rotations(&shape, key) + 1;
        reset_counts();
        let next = run(&retained, &inserted, BTreeMap::from([(key, 2)]), u64::MAX);
        reconcile((*next.report_for_test()));
        assert!(matches!((*next.report_for_test()), TreeRun::Edited(_)));
        assert_eq!(
            (*next.report_for_test())
                .metrics_for_test()
                .accounted_metrics()
                .expect("explicit accounted fixture")
                .combine_calls,
            2 * (*next.report_for_test())
                .metrics_for_test()
                .accounted_metrics()
                .expect("explicit accounted fixture")
                .recombined_nodes
        );
        assert_eq!(
            (*next.report_for_test())
                .metrics_for_test()
                .accounted_metrics()
                .expect("explicit accounted fixture")
                .recombined_nodes,
            u128::from(bound)
        );
        assert_eq!(*next.outcome_into_for_test().unwrap().result(), 130);
    }
}

#[test]
fn public_incremental_observation_reports_actual_work_apart_from_charge() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let installed = installed(StatusRead::Gather(1), counted);
    let first = first_run(&world, &installed);
    let charge = first.outcome.as_ref().unwrap().1;
    *(*installed.owner_for_test()).bump.lock().unwrap() = 10;
    reset_counts();
    let next = attempt(&world, &installed, Some(prior_of(first, true)));
    assert_eq!(next.runs, [(Run::Incremental, None)]);
    let [tree] = next.tree_runs.as_slice() else {
        panic!("one terminal report")
    };
    assert_eq!(tree.partitions_for_test(), Run::Incremental);
    reconcile(*tree);
    assert!(matches!(tree, TreeRun::Edited(_)));
    assert_eq!(next.outcome.unwrap().1, charge);
}

pub fn reset_counts() {
    CALLS.set(0);
    NODES.set(0);
    PAIR.set(false);
}

#[path = "admission/mod.rs"]
pub mod admission;
#[path = "attempts/mod.rs"]
pub mod attempts;
#[path = "causes/mod.rs"]
pub mod causes;
#[path = "mean/mod.rs"]
pub mod mean;
#[path = "shape/mod.rs"]
pub mod shape;
