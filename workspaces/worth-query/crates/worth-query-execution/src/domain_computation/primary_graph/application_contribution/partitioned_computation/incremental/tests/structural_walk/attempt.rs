//! Run under the supplied actual root request, then seal its retained computation.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::QueryRequestExecution;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::{
    program::authorization::authorize, schema::*,
};
use crate::domain_computation::primary_graph::WorthQueryManagedComputationExecution;
use worth_execution::ExecutionAllocationPolicy as Allocation;
use worth_foundational::facade::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;
pub(super) fn run(
    world: &world::World,
    installed: &Installed,
    prior: Option<ComputationPrior>,
    accounted: bool,
    cause: Cause,
) -> SealedComputationRun {
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let execution = host
                .execution_request_for(world.application.product_runtime())
                .unwrap();
            authorize(
                world,
                &phase,
                &execution,
                &request,
                0,
                InspectWalk::reference(),
                None,
            )
            .0
        })
        .unwrap();
    let policy = if accounted {
        ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, None),
        )
    } else {
        ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::MIN,
        )
    };
    let execution =
        QueryRequestExecution::open(RuntimeWorldExecutionPlacement::Serial(policy), &request);
    assert_eq!(
        execution.has_accounting(),
        accounted,
        "the real computation request uses the tested posture"
    );
    let native = execution.execution_request().unwrap();
    assert_eq!(native.has_accounting(), accounted);
    super::super::super::observed::partitioned_computation_runs_on_this_thread_for_test();
    ComputationPrior::hand_in_test(prior);
    SealedComputationRun::keep_in_test(None);
    let completed = world
        .invariant
        .project_admitted_operation_with_request(
            &native,
            &admission,
            |reader, root| {
                let prepared = installed.prepare_through(reader, &execution, root).unwrap();
                let computed = prepared
                    .compute(WorthQueryManagedComputationExecution::new(&execution))
                    .unwrap();
                assert_eq!(computed.complete().unwrap(), 4);
            },
            Allocation::SystemAllocation,
        )
        .unwrap();
    ComputationPrior::hand_in_test(None);
    let (_, projection, _) = completed.into_parts();
    world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    Allocation::SystemAllocation,
                )
                .unwrap()
                .complete_projected_dependencies(&phase, Allocation::SystemAllocation)
                .unwrap();
        })
        .unwrap();
    let runs =
        super::super::super::observed::partitioned_computation_runs_on_this_thread_for_test();
    assert!(
        matches!(runs.as_slice(),[(Run::Full(actual),_)] if *actual==cause),
        "the actual comparator chose the required full cause: {runs:?}"
    );
    SealedComputationRun::kept_in_test().expect("the real completed read sealed the owner run")
}
