use super::{fixture, *};
use crate::domain_computation::execution_runtime::WorthQueryApplicationCandidateResourceProfile as Profile;
use crate::domain_computation::primary_graph::application_entry::mutation::{
    HandlerResult, MutationHandlerExecutionDenial, WorthQueryCompletedMutationCandidate,
};
use fixture::{MeasuredBinding, MeasuredEmitOperation, MeasuredInput, ReservationSchema};
use std::sync::atomic::Ordering;
use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

fn execute(
    world: &fixture::ReservationWorld,
    input: &MeasuredInput,
) -> Result<
    HandlerResult<
        WorthQueryCompletedMutationCandidate<ReservationSchema, MeasuredBinding>,
        MeasuredInput,
    >,
    MutationHandlerExecutionDenial,
> {
    execute_with_allocation(
        world,
        input,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
}

fn execute_with_allocation(
    world: &fixture::ReservationWorld,
    input: &MeasuredInput,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    HandlerResult<
        WorthQueryCompletedMutationCandidate<ReservationSchema, MeasuredBinding>,
        MeasuredInput,
    >,
    MutationHandlerExecutionDenial,
> {
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    execute_with_request(world, input, allocation_policy, &request)
}

fn execute_with_request(
    world: &fixture::ReservationWorld,
    input: &MeasuredInput,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Result<
    HandlerResult<
        WorthQueryCompletedMutationCandidate<ReservationSchema, MeasuredBinding>,
        MeasuredInput,
    >,
    MutationHandlerExecutionDenial,
> {
    world
        .application
        .with_host_advancement(|phase| {
            let principal = fixture::authenticated_principal(&phase, world, request);
            let account = fixture::resolved_account(&phase, world, "open", request);
            let operation = world
                .application
                .installed_schema()
                .installed_operation(MeasuredEmitOperation::reference())
                .unwrap();
            let admission = world
                .selected_product(&phase)
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    TypedMutationPreconditions::new(),
                    request,
                )
                .unwrap();
            let key = "measured-source-shape".to_owned();
            let identities =
                ApplicationMutationIdentities::<ReservationSchema, MeasuredBinding>::encode(
                    &key, input,
                )
                .unwrap();
            world
                .application
                .execute_mutation_handler::<MeasuredBinding>(
                    &phase,
                    &identities,
                    &1,
                    admission,
                    allocation_policy,
                )
        })
        .expect("actual installed host phase")
}

#[test]
fn system_request_cancelled_by_requirements_hook_never_constructs_candidate() {
    use worth_query_admission::facade::authenticated_principal::{
        WorthQueryCancellationSource, WorthQueryRequestScope,
    };
    let world =
        fixture::installed_world_with_candidate_policy(Profile::policy(None, None).unwrap());
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(None, cancellation.token());
    *world.cancel_on_requirements.lock().unwrap() = Some(cancellation.clone());
    let refusal = match execute_with_request(
        &world,
        &MeasuredInput(vec![3, 5]),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        &request,
    ) {
        Err(MutationHandlerExecutionDenial::Attempt(refusal)) => refusal,
        _ => panic!("the original request stop must precede candidate allocation and writing"),
    };
    assert!(
        cancellation.token().is_cancelled(),
        "the real handler requirements hook ran"
    );
    assert_eq!(
        refusal.kind(),
        WorthQueryApplicationAttemptDenialKind::Cancelled
    );
    assert_eq!(world.candidate_contacts.load(Ordering::Relaxed), 0);
}

#[test]
fn actual_reservations_fit_separately_but_combined_host_policy_refuses() {
    for profile in [
        Profile::policy(Some(3), None).unwrap(),
        Profile::policy(None, Some(6)).unwrap(),
    ] {
        let world = fixture::installed_world_with_candidate_policy(profile);
        let first = match execute(&world, &MeasuredInput(vec![3, 5])).unwrap() {
            HandlerResult::Completed(candidate) => candidate,
            _ => panic!("first actual reservation fits individually"),
        };
        let (first, _) = first.into_parts();
        let first_demand = first.candidate_requirements.unwrap();
        drop(first);
        let second = match execute(&world, &MeasuredInput(vec![7, 11])).unwrap() {
            HandlerResult::Completed(candidate) => candidate,
            _ => panic!("second actual reservation fits individually"),
        };
        let (second, _) = second.into_parts();
        let refusal = super::super::validate_candidate_extension(
            &second.read_set.admission,
            second.candidate_requirements,
            first_demand,
        )
        .unwrap_err();
        assert_eq!(
            refusal.kind(),
            WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded
        );
        assert_eq!(world.candidate_contacts.load(Ordering::Relaxed), 2);
        drop(second);
        let fresh = crate::domain_computation::primary_graph::tests::fixture::live_scope();
        world
            .application
            .with_host_advancement(|phase| {
                fixture::resolved_account(&phase, &world, "open", &fresh);
            })
            .unwrap();
    }
}

#[test]
fn ordinary_measured_candidate_keeps_original_request_backing_until_discard() {
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let world =
        fixture::installed_world_with_candidate_policy(Profile::policy(None, None).unwrap());
    let parent = worth_execution::ExecutionResourceLease::serial(worth_execution::LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, None),
        ),
        cancellation: worth_execution::CancellationToken::new(),
        deadline: None,
    })
    .unwrap();
    let input = MeasuredInput(vec![3, 5, 7]);
    let completed = match execute_with_allocation(
        &world,
        &input,
        worth_execution::ExecutionAllocationPolicy::Execution(&parent),
    )
    .unwrap()
    {
        HandlerResult::Completed(completed) => completed,
        _ => panic!("the actual original request must fund the candidate"),
    };
    let (program, _) = completed.into_parts();
    assert!(program.effect_backing_memory.is_some());
    assert!(
        parent.charged_memory_bytes()
            >= (input.0.len()
                * std::mem::size_of::<super::super::WorthQueryApplicationRealizedEffect>())
                as u64
    );
    drop(program);
    assert_eq!(parent.charged_memory_bytes(), 0);
}

#[test]
fn ordinary_handler_reserves_two_actual_shapes_without_declaration_or_host_ceiling() {
    let world =
        fixture::installed_world_with_candidate_policy(Profile::policy(None, None).unwrap());
    for source in [vec![3, 5], vec![7, 11, 13, 17, 19]] {
        let input = MeasuredInput(source);
        let expected = input.0.len();
        let completed = match execute(&world, &input).unwrap() {
            HandlerResult::Completed(completed) => completed,
            _ => panic!("measured source shape must construct a candidate"),
        };
        let (program, result) = completed.into_parts();
        assert_eq!(result, input);
        assert_eq!(program.effects.len(), expected);
        // The real program is deliberately discarded; its source account survives.
        drop(program);
    }
    assert_eq!(world.candidate_contacts.load(Ordering::Relaxed), 2);
}

#[test]
fn explicit_host_count_and_representation_policies_refuse_before_candidate_construction() {
    for profile in [
        Profile::policy(Some(2), None).unwrap(),
        Profile::policy(None, Some(4)).unwrap(),
    ] {
        let world = fixture::installed_world_with_candidate_policy(profile);
        let refusal = match execute(&world, &MeasuredInput(vec![3, 5, 7])) {
            Err(MutationHandlerExecutionDenial::Attempt(refusal)) => refusal,
            _ => panic!("actual demand must meet the supplied finite host policy"),
        };
        assert_eq!(
            refusal.kind(),
            WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded
        );
        assert_eq!(world.candidate_contacts.load(Ordering::Relaxed), 0);
        let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
        world
            .application
            .with_host_advancement(|phase| {
                fixture::resolved_account(&phase, &world, "open", &request);
            })
            .unwrap();
    }
}
