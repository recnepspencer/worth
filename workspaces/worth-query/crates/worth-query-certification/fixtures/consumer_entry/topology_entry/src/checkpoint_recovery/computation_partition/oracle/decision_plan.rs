//! Captures preparation and compute probes at their actual phase boundaries.
use super::*;
use std::cell::RefCell;
use std::rc::Rc;
use worth_query_consumer_values::PlanarMutationDenial;
use worth_query_host::facade::{
    application_contribution::*,
    primary_graph::{
        full_partitioned_computation_preparations_on_this_thread_for_test,
        partitioned_computation_tree_work_on_this_thread_for_test, HandlerResult,
        WorthQueryDecisionPlan,
    },
};

pub(super) fn begin() {
    full_partitioned_computation_preparations_on_this_thread_for_test();
    take_calls();
}

pub(super) fn prepared<Computation, Owner>(
    prepared: Result<
        WorthQueryPreparedPartitionedComputation<
            CheckpointSchema,
            PlanarFinalOutputFeature,
            Computation,
            Owner,
        >,
        WorthQueryPartitionedComputationDenial<u32>,
    >,
    reports: impl FnOnce(Vec<Option<u64>>) + 'static,
) -> WorthQueryDecisionPlan<Option<u64>, PlanarMutationDenial>
where
    Computation: ApplicationManagedComputation<CheckpointSchema, PlanarFinalOutputFeature>,
    Owner: WorthQueryPartitionedComputationOwner<
        CheckpointSchema,
        PlanarFinalOutputFeature,
        Computation,
        Output = f64,
        Stopped = u32,
    >,
{
    let calls = take_calls();
    let preparations = full_partitioned_computation_preparations_on_this_thread_for_test();
    type ComputeObservation = (
        Vec<(
            WorthQueryPartitionedComputationRunForTest,
            Option<worth_foundational::facade::ExecutionReport>,
        )>,
        Vec<WorthQueryPartitionedTreeRun>,
        OwnerCalls,
        usize,
        usize,
        worth_query_host::facade::primary_graph::WorthQueryExecutionPlacementForTest,
    );
    let observation: Rc<RefCell<Option<ComputeObservation>>> = Rc::new(RefCell::new(None));
    let completing = observation.clone();
    let finish = move |computed: Result<
        WorthQueryCompletedPartitionedComputation<
            CheckpointSchema,
            PlanarFinalOutputFeature,
            Computation,
            Owner,
        >,
        WorthQueryPartitionedComputationDenial<u32>,
    >| {
        let outcome = computed.and_then(|computed| {
            let charge = computed.charged_work();
            computed.complete().map(|value| (value.to_bits(), charge))
        });
        let value = outcome.as_ref().ok().map(|(bits, _)| *bits);
        let (runs, trees, computed_calls, combines, nodes, placement) =
            completing.borrow_mut().take().unwrap_or_else(|| {
                (
                    Vec::new(),
                    Vec::new(),
                    OwnerCalls::default(),
                    0,
                    0,
                    tree_count::placement(),
                )
            });
        reports(
            runs.iter()
                .map(|(_, report)| report.map(|report| report.charged_work()))
                .collect(),
        );
        room().push(OracleRun {
            published: Vec::new(),
            outcome,
            full_preparations: preparations,
            runs: runs.into_iter().map(|(run, _)| run).collect(),
            tree_runs: trees,
            calls: OwnerCalls {
                plans: calls.plans + computed_calls.plans,
                keys: calls.keys + computed_calls.keys,
                gathers: calls.gathers + computed_calls.gathers,
                kernels: calls.kernels + computed_calls.kernels,
            },
            combines,
            tree_nodes: nodes,
            placement,
        });
        HandlerResult::Completed(value)
    };
    match prepared {
        Err(error) => WorthQueryDecisionPlan::immediate(finish(Err(error))),
        Ok(prepared) => prepared
            .into_decision_plan(finish)
            .observe_computes_for_test(
                || {
                    runs_on_this_thread();
                    partitioned_computation_tree_work_on_this_thread_for_test();
                    take_calls();
                    COMBINES.store(0, Ordering::Relaxed);
                    tree_count::reset();
                },
                || {
                    (
                        runs_on_this_thread(),
                        partitioned_computation_tree_work_on_this_thread_for_test(),
                        take_calls(),
                        COMBINES.swap(0, Ordering::Relaxed),
                        tree_count::take_nodes(),
                        tree_count::placement(),
                    )
                },
                move |mut observations| {
                    assert_eq!(observations.len(), 1, "one request's compute observation");
                    *observation.borrow_mut() = observations.pop();
                },
            ),
    }
}
