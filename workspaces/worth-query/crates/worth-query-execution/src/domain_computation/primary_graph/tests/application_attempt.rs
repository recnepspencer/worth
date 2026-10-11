#[cfg(feature = "test-query-execution-observer")]
#[path = "application_attempt/advancement_custody.rs"]
mod advancement_custody;
#[path = "application_attempt/product_races.rs"]
mod product_races;

use std::time::Duration;

use super::fixture::{
    installed_authorization_world, installed_program_support, live_scope, AccountStatus,
    ProgramRequiredInput, ProgramRequiredOperation, TouchAccountOperation,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitTerminalKind,
    WorthQueryApplicationIdempotencyBinding, WorthQueryPrincipalResolutionMode,
};
#[path = "application_attempt/authority_lifecycle.rs"]
mod authority_lifecycle;
#[path = "application_attempt/authorization_causality.rs"]
mod authorization_causality;
#[path = "application_attempt/decision_adjacency.rs"]
mod decision_adjacency;
#[path = "application_attempt/decision_manifest_ceiling.rs"]
mod decision_manifest_ceiling;
#[path = "application_attempt/effect_authority.rs"]
mod effect_authority;
#[path = "application_attempt/emitted_effects.rs"]
mod emitted_effects;
#[path = "application_attempt/guarded_operation_custody.rs"]
mod guarded_operation_custody;
#[path = "application_attempt/host_commit_identity.rs"]
mod host_commit_identity;
#[path = "application_attempt/idempotency_behavior.rs"]
mod idempotency_behavior;
#[path = "application_attempt/index_budget.rs"]
mod index_budget;
#[path = "application_attempt/live_delivery_capacity.rs"]
mod live_delivery_capacity;
#[path = "application_attempt/mutation_binding_identity.rs"]
mod mutation_binding_identity;
#[path = "application_attempt/mutation_terminal_lifecycle.rs"]
mod mutation_terminal_lifecycle;
#[path = "application_attempt/mutation_work_scale.rs"]
mod mutation_work_scale;
#[path = "application_attempt/optional_field_mutation.rs"]
mod optional_field_mutation;
#[path = "application_attempt/optional_output_role.rs"]
mod optional_output_role;
#[path = "application_attempt/post_commit_recovery.rs"]
mod post_commit_recovery;
#[path = "application_attempt/predecode_admission.rs"]
mod predecode_admission;
#[path = "application_attempt/preimage_evidence.rs"]
pub(in crate::domain_computation::primary_graph) mod preimage_evidence;
#[path = "application_attempt/preimage_retention.rs"]
mod preimage_retention;
#[path = "application_attempt/producer_invariant_publication.rs"]
mod producer_invariant_publication;
#[path = "application_attempt/program_fixture.rs"]
mod program_fixture;
pub(in crate::domain_computation::primary_graph) use program_fixture::admitted_operation;
#[path = "application_attempt/native_preparation.rs"]
mod native_preparation;
#[path = "application_attempt/program_lane.rs"]
mod program_lane;
#[path = "application_attempt/program_occurrence_gate.rs"]
mod program_occurrence_gate;
#[path = "application_attempt/provider_terminal_evidence.rs"]
mod provider_terminal_evidence;
#[path = "application_attempt/retry_outbox_rebind.rs"]
mod retry_outbox_rebind;
#[path = "application_attempt/settlement_failures.rs"]
mod settlement_failures;
#[path = "application_attempt/structured_effect_validation.rs"]
mod structured_effect_validation;
#[path = "application_attempt/terminal_failures.rs"]
mod terminal_failures;
#[path = "application_attempt/touched_graph_closure.rs"]
mod touched_graph_closure;
#[cfg(feature = "test-world-operation-control")]
#[path = "application_attempt/unwind_custody.rs"]
mod unwind_custody;
#[path = "application_attempt/validation_interruption.rs"]
mod validation_interruption;

use program_fixture::{
    admitted_mutation_free_program, admitted_program, admitted_program_on_selected,
    admitted_program_with_emit, admitted_program_with_expected_status,
};

#[path = "application_attempt/decision_currentness.rs"]
mod decision_currentness;
pub(in crate::domain_computation::primary_graph) use decision_currentness::{
    assert_changed_decision, assert_product_basis_stale,
};

#[test]
fn concurrent_equivalent_attempts_publish_one_transaction() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let first = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(&phase, &world, &principal, &account, &request, "once")
        })
        .expect("the fixture read owns its installed host call");
    let second = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(&phase, &world, &principal, &account, &request, "once")
        })
        .expect("the fixture read owns its installed host call");

    let (left, right) = std::thread::scope(|scope| {
        let left = scope.spawn(|| {
            world.application.compare_and_commit_application(
                first,
                idempotency(12, 12),
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
        });
        let right = scope.spawn(|| {
            world.application.compare_and_commit_application(
                second,
                idempotency(12, 12),
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
        });
        (left.join().unwrap(), right.join().unwrap())
    });
    let receipts = [left, right]
        .into_iter()
        .map(|outcome| match outcome {
            WorthQueryApplicationCommitOutcome::Committed(receipt)
            | WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) => receipt,
            unexpected => panic!("unexpected concurrent outcome: {unexpected:?}"),
        })
        .collect::<Vec<_>>();
    assert!(receipts[0].is_same_authoritative_commit(&receipts[1]));
    let terminal_kinds = receipts
        .iter()
        .map(|receipt| receipt.terminal().kind())
        .collect::<Vec<_>>();
    assert!(terminal_kinds.contains(&WorthQueryApplicationCommitTerminalKind::Executed));
    assert!(terminal_kinds.contains(&WorthQueryApplicationCommitTerminalKind::Recovered));
}

mod response_loss_resolves_the_published_commit_before_returning;

#[test]
fn preparation_commit_recovery_and_retry_perform_no_execution_digest_derivation() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let before = crate::execution_digest::test_hash_parts_call_count();
    let first = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "digest-free",
            )
        })
        .expect("the fixture read owns its installed host call");
    let retry = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "digest-free",
            )
        })
        .expect("the fixture read owns its installed host call");

    world.faults.lose_next_commit_response();
    assert!(matches!(
        world.application.compare_and_commit_application(
            first,
            idempotency(31, 31),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let after_commit = crate::execution_digest::test_hash_parts_call_count();
    assert_eq!(
        after_commit,
        before,
        "preparation, provider execution, commit, or response-loss recovery derived legacy execution digests: {:?}",
        crate::execution_digest::test_hash_parts_domains_since(before)
    );

    assert!(matches!(
        world.application.compare_and_commit_application(
            retry,
            idempotency(31, 31),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
    ));
    assert_eq!(
        crate::execution_digest::test_hash_parts_call_count(),
        after_commit,
        "idempotent retry or recovery derived a legacy execution digest"
    );
}

pub(in crate::domain_computation::primary_graph) fn idempotency(
    key: u8,
    intent: u8,
) -> WorthQueryApplicationIdempotencyBinding {
    WorthQueryApplicationIdempotencyBinding::new([key; 32], [intent; 32])
}

pub(in crate::domain_computation::primary_graph) fn world_issued_unpublished_material(
    seed: u8,
) -> (
    worth_runtime_world::facade::ProductBranchObservation,
    WorthQueryApplicationIdempotencyBinding,
    worth_runtime_world::facade::ProductUnpublishedRecoveryHandle,
) {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let binding = idempotency(seed, seed);
    let program = world
        .application
        .with_host_advancement(|phase| {
            let phase_custody = &phase.test_host_request();
            let principal =
                authenticated_principal(&(phase_custody).test_host_request(), &world, &request);
            let account = resolved_account(
                &(phase_custody).test_host_request(),
                &world,
                "open",
                &request,
            );
            let program = admitted_program(
                &phase_custody.phase_for_application(&world.application),
                &world,
                &principal,
                &account,
                &request,
                "bounded-unpublished-material",
            );
            world
                .application
                .fail_next_durable_append_for_test(&phase_custody.borrowed_request_for_test());

            program
        })
        .expect("fixture preparation owns its host call");
    let outcome = world.application.compare_and_commit_application(
        program,
        binding,
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::ProductUnpublished(partial) = outcome else {
        panic!("the real World must issue unpublished recovery material: {outcome:?}")
    };
    let observation = partial.expected_product().clone();
    let handle = partial.into_recovery().record_handle().clone();
    (observation, binding, handle)
}

pub(in crate::domain_computation::primary_graph) fn authenticated_principal(
    phase: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    world: &super::fixture::AuthorizationWorld,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal<
    super::fixture::IdentityExecutionSchema,
    super::fixture::Principal,
    u64,
> {
    let external = world.authenticate("alice", Duration::from_secs(60), request);
    let execution = phase
        .execution_request_for(world.application.product_runtime())
        .expect("fixture lends its installed phase");
    world
        .application
        .select_product_branch(
            &execution,
            world.application.product_runtime().default_branch(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &phase.phase_for_application(&world.application),
            &world.binding,
            &external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}

pub(in crate::domain_computation::primary_graph) fn resolved_account(
    phase: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    world: &super::fixture::AuthorizationWorld,
    status: &str,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<
    super::fixture::IdentityExecutionSchema,
    super::fixture::Account,
> {
    let execution = phase
        .execution_request_for(world.application.product_runtime())
        .expect("fixture lends its installed phase");
    world
        .application
        .select_product_branch(
            &execution,
            world.application.product_runtime().default_branch(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            &phase.phase_for_application(&world.application),
            AccountStatus::reference(),
            status.to_string(),
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}

#[path = "application_attempt/read_rendezvous.rs"]
mod read_rendezvous;

#[path = "application_attempt/request_read_stops.rs"]
mod request_read_stops;
