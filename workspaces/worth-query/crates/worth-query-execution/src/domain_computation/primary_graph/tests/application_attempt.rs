#[path = "application_attempt/product_races.rs"]
mod product_races;

use std::time::Duration;

use super::fixture::{
    installed_authorization_world, live_scope, AccountStatus, TouchAccountOperation,
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
#[path = "application_attempt/idempotency_behavior.rs"]
mod idempotency_behavior;
#[path = "application_attempt/mutation_terminal_lifecycle.rs"]
mod mutation_terminal_lifecycle;
#[path = "application_attempt/mutation_work_scale.rs"]
mod mutation_work_scale;
#[path = "application_attempt/optional_field_mutation.rs"]
mod optional_field_mutation;
#[path = "application_attempt/post_commit_recovery.rs"]
mod post_commit_recovery;
#[path = "application_attempt/preimage_evidence.rs"]
pub(in crate::domain_computation::primary_graph) mod preimage_evidence;
#[path = "application_attempt/preimage_retention.rs"]
mod preimage_retention;
#[path = "application_attempt/program_fixture.rs"]
mod program_fixture;
#[path = "application_attempt/provider_terminal_evidence.rs"]
mod provider_terminal_evidence;
#[path = "application_attempt/retry_outbox_rebind.rs"]
mod retry_outbox_rebind;
#[path = "application_attempt/settlement_failures.rs"]
mod settlement_failures;
#[path = "application_attempt/terminal_failures.rs"]
mod terminal_failures;
#[path = "application_attempt/touched_graph_closure.rs"]
mod touched_graph_closure;
#[cfg(feature = "test-world-operation-control")]
#[path = "application_attempt/unwind_custody.rs"]
mod unwind_custody;

use program_fixture::{
    admitted_mutation_free_program, admitted_program, admitted_program_with_emit,
    admitted_program_with_expected_status,
};

pub(in crate::domain_computation::primary_graph) fn assert_product_basis_stale(
    outcome: WorthQueryApplicationCommitOutcome,
    cause: &str,
) {
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("{cause} must deny before effects: {outcome:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProductBasisStale
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
}

#[test]
fn concurrent_equivalent_attempts_publish_one_transaction() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let first = admitted_program(&world, &principal, &account, &request, "once");
    let second = admitted_program(&world, &principal, &account, &request, "once");

    let (left, right) = std::thread::scope(|scope| {
        let left = scope.spawn(|| {
            world
                .application
                .compare_and_commit_application(first, idempotency(12, 12))
        });
        let right = scope.spawn(|| {
            world
                .application
                .compare_and_commit_application(second, idempotency(12, 12))
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

#[test]
fn response_loss_resolves_the_published_commit_before_returning() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let first = admitted_program_with_expected_status(
        &world,
        &principal,
        &account,
        &request,
        ("open", "published"),
    );
    let retry = admitted_program_with_expected_status(
        &world,
        &principal,
        &account,
        &request,
        ("open", "published"),
    );

    world.faults.lose_next_commit_response();
    let WorthQueryApplicationCommitOutcome::Committed(first_receipt) = world
        .application
        .compare_and_commit_application(first, idempotency(15, 15))
    else {
        panic!("authoritative idempotency must prove the response-lost commit");
    };
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) = world
        .application
        .compare_and_commit_application(retry, idempotency(15, 15))
    else {
        panic!("retry must recover the transaction published before response loss");
    };
    assert!(receipt.is_same_authoritative_commit(&first_receipt));
    assert_eq!(receipt.outcome_identity(), first_receipt.outcome_identity());
    assert_eq!(
        first_receipt.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Executed
    );
    assert_eq!(
        first_receipt.terminal().attempt_resources_released(),
        Some(true)
    );
    assert_eq!(
        receipt.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Recovered
    );
    assert_eq!(
        receipt.precondition_comparison().expected_version_count(),
        0
    );
    assert_eq!(receipt.precondition_comparison().expected_fact_count(), 1);
    assert!(receipt.changed_record_count() >= 2);
}

#[test]
fn preparation_commit_recovery_and_retry_perform_no_execution_digest_derivation() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let before = crate::execution_digest::test_hash_parts_call_count();
    let first = admitted_program(&world, &principal, &account, &request, "digest-free");
    let retry = admitted_program(&world, &principal, &account, &request, "digest-free");

    world.faults.lose_next_commit_response();
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(first, idempotency(31, 31)),
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
        world
            .application
            .compare_and_commit_application(retry, idempotency(31, 31)),
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
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let binding = idempotency(seed, seed);
    let program = admitted_program(
        &world,
        &principal,
        &account,
        &request,
        "bounded-unpublished-material",
    );
    world.application.fail_next_durable_append_for_test();
    let outcome = world
        .application
        .compare_and_commit_application(program, binding);
    let WorthQueryApplicationCommitOutcome::ProductUnpublished(partial) = outcome else {
        panic!("the real World must issue unpublished recovery material: {outcome:?}")
    };
    let observation = partial.expected_product().clone();
    let handle = partial.into_recovery().record_handle().clone();
    (observation, binding, handle)
}

pub(in crate::domain_computation::primary_graph) fn authenticated_principal(
    world: &super::fixture::AuthorizationWorld,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal<
    super::fixture::IdentityExecutionSchema,
    super::fixture::Principal,
    u64,
> {
    let external = world.authenticate("alice", Duration::from_secs(60), request);
    world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}

pub(in crate::domain_computation::primary_graph) fn resolved_account(
    world: &super::fixture::AuthorizationWorld,
    status: &str,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<
    super::fixture::IdentityExecutionSchema,
    super::fixture::Account,
> {
    world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            status.to_string(),
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}
