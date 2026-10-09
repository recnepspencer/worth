//! Budgets derived from the owned map's declared storage, never a measured run.
use super::super::super::{PreparedWorkflowFrontier, WorkflowStagePreparation};
use super::*;
use worth_execution::{ChargedBytes, ExecutionMap};
use worth_foundational::PartitionIdentity;

pub(super) fn start_reservation(probe: &phased_workflow::fold_executor::ComputeProbe) -> u64 {
    let mut workspace = Policy::width(0).workspace(probe);
    let run = start(&mut workspace);
    let declaration = run.prepare_frontier_computation(vec![(
        "start".into(),
        domain::WorthQueryWorkflowValue::NotRequired,
    )]);
    let authority = probe.authority();
    // This lease is queried only for the owner's framework declaration. It is
    // never passed to an executing door or used to fund the fixture's work.
    let lease = authority
        .request_lease(worth_execution::LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, 64 * 1024 * 1024, 1024),
            ),
            cancellation: worth_execution::CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    let root = framework(&lease, 1);
    workspace
        .advancement_owner()
        .with_advancement(|_phase| {
            // Serial root has one token; its nested request scope and map each
            // inherit that token and add the same request's token once. Neither
            // allocates another physical activity cell (run_context.rs).
            let nested = framework(&lease, 1);
            root + nested + nested + declared_reservation(declaration)
        })
        .unwrap()
}

pub(super) fn declared_reservation(frontier: PreparedWorkflowFrontier) -> u64 {
    let count = frontier.members.len();
    let mut input = 0;
    let mut scratch = 0;
    let mut result = 0;
    for member in frontier.members.into_values() {
        let WorkflowStagePreparation::Ready(task) = member.preparation else {
            panic!("the declared fixture frontier prepares every member")
        };
        input += task.additional_charged_bytes();
        scratch += task.scratch_bytes();
        result += task.result_capacity().unwrap();
    }
    // Keyless maps retain one empty read vector and one empty write member
    // per partition, as declared by ExecutionMap::from_keyless_partitions.
    let access = count
        * (std::mem::size_of::<Vec<()>>() + std::mem::size_of::<(PartitionIdentity, Vec<()>)>());
    ExecutionMap::<domain::WorthQueryWorkflowStageTask, ()>::declared_memory_requirement::<
        domain::WorthQueryWorkflowStageComputed,
        domain::WorthQueryWorkflowStageComputed,
    >(count, input, scratch, result, access as u64)
    .unwrap()
}

pub(super) fn framework(lease: &worth_execution::ExecutionResourceLease<'_>, count: usize) -> u64 {
    ExecutionMap::<(), ()>::declared_memory_requirement_for_lease::<(), ()>(
        lease, count, 0, 0, 0, 0,
    )
    .unwrap()
        - ExecutionMap::<(), ()>::declared_memory_requirement::<(), ()>(count, 0, 0, 0, 0).unwrap()
}
