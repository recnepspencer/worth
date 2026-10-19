//! Ordinary installed preparation, publication and independent current reads.
use super::fixture::{self, prepared::*, *};
use crate::domain_computation::primary_graph::*;
use std::sync::atomic::Ordering;
use worth_execution::ExecutionAllocationPolicy;
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationIdentities,
    application_schema::TypedMutationPreconditions,
};

fn world(items: u64, bytes: u64) -> ReservationWorld {
    fixture::installed_world_with_profile(crate::domain_computation::execution_runtime::WorthQueryApplicationCandidateResourceProfile::physical_resources(items, bytes).unwrap())
}
fn input(key: &str, count: usize, width: usize) -> PreparedInput {
    PreparedInput {
        key: key.into(),
        values: (0..count)
            .map(|i| format!("{key}-{i}-{}", "x".repeat(width)))
            .collect(),
        declared: count,
        plan: MemberPlan::Create,
    }
}
fn execute<const FIXED: bool>(
    world: &ReservationWorld,
    input: &PreparedInput,
    allocation: ExecutionAllocationPolicy<'_, '_>,
) -> (
    Result<
        HandlerResult<
            WorthQueryCompletedMutationCandidate<ReservationSchema, PreparedBinding<FIXED>>,
            PreparedInput,
        >,
        MutationHandlerExecutionDenial,
    >,
    WorthQueryApplicationIdempotencyBinding,
) {
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    world
        .application
        .with_application_advancement(&request, |phase| {
            let principal = fixture::authenticated_principal(&phase, world, &request);
            let root = fixture::resolved_account(&phase, world, "open", &request);
            let operation = world
                .application
                .installed_schema()
                .installed_operation(PreparedOperation::reference())
                .unwrap();
            let admission = world
                .selected_product(&phase)
                .authorize_operation(
                    &phase,
                    &principal,
                    &root,
                    &operation,
                    TypedMutationPreconditions::new(),
                    &request,
                )
                .unwrap();
            let identities = ApplicationMutationIdentities::<
                ReservationSchema,
                PreparedBinding<FIXED>,
            >::encode(&input.key, input)
            .unwrap();
            let key = WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities);
            let report = world
                .application
                .execute_mutation_handler_report::<PreparedBinding<FIXED>>(
                    &phase,
                    &identities,
                    principal.principal_identity(),
                    admission,
                    allocation,
                );
            (report.into_outcome(), key)
        })
        .unwrap()
}

#[test]
fn prepared_contract_publishes_two_real_footprints_with_required_family() {
    let world = world(32, 131_072);
    let funding = lease(16 * 1024 * 1024);
    let policy = ExecutionAllocationPolicy::Execution(&funding);
    for authored in [input("small", 1, 8), input("large", 3, 8192)] {
        let (outcome, key) = execute::<false>(&world, &authored, policy);
        let Ok(HandlerResult::Completed(completed)) = outcome else {
            panic!("prepared footprint refused");
        };
        let (program, result) = completed.into_parts();
        assert_eq!(result, authored);
        let committed = match world
            .application
            .compare_and_commit_application(program, key, policy)
        {
            WorthQueryApplicationCommitOutcome::Committed(receipt) => receipt,
            other => panic!("native publication refused: {other:?}"),
        };
        let outputs = committed
            .output_correspondence()
            .outputs_of::<PreparedOutputs>()
            .unwrap();
        let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
        world
            .application
            .with_host_advancement(|phase| {
                let root = fixture::resolved_account(&phase, &world, "open", &request);
                assert_eq!(
                    outputs.entity::<PreparedRoot>().unwrap().entity_id(),
                    root.entity_id()
                );
                let members = outputs.family_entries::<PreparedMembers>().unwrap();
                assert_eq!(members.len(), authored.values.len());
                for (i, expected) in authored.values.iter().enumerate() {
                    let actual = fixture::resolved_account(&phase, &world, expected, &request);
                    assert_eq!(members[i].entity_id(), actual.entity_id());
                }
            })
            .unwrap();
    }
    assert_eq!(world.authored.load(Ordering::SeqCst), 2);
}

#[test]
fn actual_runtime_and_fixed_counts_refuse_before_candidate_authoring() {
    let input = input("denied", 2, 1024);
    for world in [world(3, 16_384), world(32, 1024)] {
        let original = source_observation(&world);
        let (outcome, _) =
            execute::<false>(&world, &input, ExecutionAllocationPolicy::SystemAllocation);
        assert!(
            matches!(outcome, Err(MutationHandlerExecutionDenial::Attempt(ref cause)) if cause.kind() == WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded)
        );
        assert_eq!(world.authored.load(Ordering::SeqCst), 0);
        assert_eq!(source_observation(&world), original);
    }
    let world = world(32, 16_384);
    let original = source_observation(&world);
    let (outcome, _) = execute::<true>(&world, &input, ExecutionAllocationPolicy::SystemAllocation);
    assert!(
        matches!(outcome, Err(MutationHandlerExecutionDenial::Attempt(ref cause)) if cause.kind() == WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded)
    );
    assert_eq!(world.authored.load(Ordering::SeqCst), 0);
    let mut underdeclared = input.clone();
    underdeclared.declared = 1;
    let (outcome, _) = execute::<false>(
        &world,
        &underdeclared,
        ExecutionAllocationPolicy::SystemAllocation,
    );
    let Err(MutationHandlerExecutionDenial::Handler(cause)) = outcome else {
        panic!("underdeclared writer must refuse");
    };
    assert_eq!(
        cause
            .downcast::<WorthQueryApplicationAttemptDenial>()
            .unwrap()
            .kind(),
        WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded
    );
    assert_eq!(world.authored.load(Ordering::SeqCst), 1);
    assert_eq!(source_observation(&world), original);
}

#[test]
fn allocation_refuses_before_authoring_and_candidate_grant_survives_sealing() {
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let world = world(32, 4 * 1024 * 1024);
    let lease = worth_execution::ExecutionResourceLease::serial(worth_execution::LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, 1024 * 1024, 1_000_000),
        ),
        cancellation: worth_execution::CancellationToken::new(),
        deadline: None,
    })
    .unwrap();
    let policy = ExecutionAllocationPolicy::Execution(&lease);
    let (outcome, _) = execute::<false>(&world, &input("unfunded", 1, 768 * 1024), policy);
    let Err(MutationHandlerExecutionDenial::Attempt(cause)) = outcome else {
        panic!("large prepared allocation must refuse before authoring");
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationAttemptDenialKind::AllocationDenied
    );
    assert_eq!(cause.subject(), "prepared candidate");
    assert_eq!(world.authored.load(Ordering::SeqCst), 0);
    let baseline = lease.charged_memory_bytes();
    let (outcome, _) = execute::<false>(&world, &input("funded", 1, 32), policy);
    let Ok(HandlerResult::Completed(completed)) = outcome else {
        panic!("funded retry failed");
    };
    let (program, _) = completed.into_parts();
    let funding = program.candidate_funding.as_ref().unwrap().clone();
    let weak = std::sync::Arc::downgrade(&funding);
    assert!(lease.charged_memory_bytes() > baseline);
    drop(program);
    assert!(weak.upgrade().is_some());
    drop(funding);
    assert!(weak.upgrade().is_none());
    assert_eq!(lease.charged_memory_bytes(), baseline);
}

fn lease(bytes: u64) -> worth_execution::ExecutionResourceLease<'static> {
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    worth_execution::ExecutionResourceLease::serial(worth_execution::LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, bytes, 1_000_000),
        ),
        cancellation: worth_execution::CancellationToken::new(),
        deadline: None,
    })
    .unwrap()
}

fn source_observation(
    world: &ReservationWorld,
) -> worth_runtime_world::facade::ProductBranchObservation {
    world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&phase)
                .product()
                .observation()
                .clone()
        })
        .unwrap()
}

#[test]
fn preserved_family_entries_with_zero_effect_demand_keep_funded_sealed_storage() {
    let world = world(1, 4096);
    let payer = lease(1024 * 1024);
    let policy = ExecutionAllocationPolicy::Execution(&payer);
    let authored = PreparedInput {
        key: "preserved".to_owned(),
        values: (0..64).map(|i| format!("alias-{i}")).collect(),
        declared: 0,
        plan: MemberPlan::Preserve,
    };
    let (outcome, key) = execute::<false>(&world, &authored, policy);
    let Ok(HandlerResult::Completed(completed)) = outcome else {
        panic!("preserved family candidate completes");
    };
    let (program, _) = completed.into_parts();
    let held = std::sync::Arc::downgrade(program.candidate_funding.as_ref().unwrap());
    let receipt = match world
        .application
        .compare_and_commit_application(program, key, policy)
    {
        WorthQueryApplicationCommitOutcome::Committed(receipt) => receipt,
        other => panic!("native preserve publication failed: {other:?}"),
    };
    let cloned = receipt.output_correspondence().clone();
    let outputs = cloned.outputs_of::<PreparedOutputs>().unwrap();
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    world.application.with_host_advancement(|phase| {
        let root = fixture::resolved_account(&phase, &world, "open", &request);
        assert_eq!(outputs.entity::<PreparedRoot>().unwrap().entity_id(), root.entity_id());
        assert_eq!(outputs.family_entries::<PreparedMembers>().unwrap().len(), authored.values.len());
        for suffix in &authored.values {
            assert_eq!(outputs.member::<PreparedMembers, worth_query_declaration::facade::application_operation::WorthQueryPreserveOutput>(suffix).unwrap().entity_id(), root.entity_id());
        }
    }).unwrap();
    drop(receipt);
    assert!(
        held.upgrade().is_some(),
        "cloned immutable correspondence retains original funding"
    );
    drop(cloned);
    // Native publication may retain the same correspondence in its receipt index;
    // the direct funding-owner test proves release after the final actual owner.
}
