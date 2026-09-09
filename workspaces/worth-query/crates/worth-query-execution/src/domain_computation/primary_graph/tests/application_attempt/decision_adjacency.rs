use std::time::Duration;

#[path = "decision_adjacency/product_currentness.rs"]
mod product_currentness;

use super::super::fixture::{
    AccountOwner, AccountStatus, ChangeOwnershipInput, ChangeOwnershipOperation,
    IdentityExecutionSchema, Principal, PrincipalIdentityField, TouchAccountOperation,
};
use super::{idempotency, installed_authorization_world, live_scope, resolved_account};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationEntityIdentity, WorthQueryAuthenticatedPrincipal,
    WorthQueryInvariantProjectionTraversalDenialKind, WorthQueryPrincipalResolutionMode,
};

#[test]
fn an_edge_entering_an_observed_empty_adjacency_stales_the_attempt() {
    let world = installed_authorization_world(false);
    let request = live_scope();
    let actor = authenticated(&world, "alice", &request);
    let principal = resolved_principal(&world, 1, &request);
    let account = resolved_account(&world, "open", &request);
    let losing = link_program(
        &world, &actor, &principal, &account, &request, "open", "losing",
    );
    let winner = link_program(
        &world, &actor, &principal, &account, &request, "open", "winner",
    );

    assert!(matches!(
        world
            .application
            .compare_and_commit_application(winner, idempotency(31, 31)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let outcome = world
        .application
        .compare_and_commit_application(losing, idempotency(32, 32));
    assert_product_basis_stale(outcome, "the edge entering the sealed empty adjacency");
}

#[test]
fn sealed_adjacency_membership_supplies_exact_unlink_evidence() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let actor = authenticated(&world, "alice", &request);
    let principal = resolved_principal(&world, 1, &request);
    let account = resolved_account(&world, "open", &request);
    let program = unlink_program(&world, &actor, &principal, &account, &request);

    let WorthQueryApplicationCommitOutcome::Committed(_) = world
        .application
        .compare_and_commit_application(program, idempotency(35, 35))
    else {
        panic!("the exact relation carried by adjacency evidence must be unlinkable");
    };
    assert_membership_absent(&world, &actor, &principal, &account, &request);
}

#[test]
fn removing_an_observed_present_relation_stales_a_competing_program() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let actor = authenticated(&world, "alice", &request);
    let principal = resolved_principal(&world, 1, &request);
    let account = resolved_account(&world, "open", &request);
    let winner = unlink_program(&world, &actor, &principal, &account, &request);
    let loser = unlink_program(&world, &actor, &principal, &account, &request);

    assert!(matches!(
        world
            .application
            .compare_and_commit_application(winner, idempotency(36, 36)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let outcome = world
        .application
        .compare_and_commit_application(loser, idempotency(37, 37));
    assert_product_basis_stale(outcome, "removing the retained relation");
    assert_membership_absent(&world, &actor, &principal, &account, &request);
}

fn assert_product_basis_stale(outcome: WorthQueryApplicationCommitOutcome, cause: &str) {
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
fn compile_capability_cannot_widen_the_installed_relation_manifest() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let actor = authenticated(&world, "alice", &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(&actor, &account, &operation, Default::default(), &request)
        .unwrap();
    let projected = world
        .invariant
        .project_admitted_operation(&admission, |reader, account| {
            reader.decision_relations_to(AccountOwner::reference(), account)
        })
        .unwrap();
    let denial = projected
        .output()
        .as_ref()
        .expect_err("an uninstalled relation target must be denied");

    assert_eq!(
        denial.kind(),
        WorthQueryInvariantProjectionTraversalDenialKind::UndeclaredDecisionTarget
    );
}

#[test]
fn capability_relation_traversal_does_not_enter_sealed_decision_dependencies() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let actor = authenticated(&world, "alice", &request);
    let principal = resolved_principal(&world, 1, &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ChangeOwnershipOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(&actor, &principal, &operation, Default::default(), &request)
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, principal| {
            reader
                .relations_from(AccountOwner::reference(), principal)
                .unwrap();
            let account = reader
                .resolve_entity(AccountStatus::reference(), "open".to_string())
                .unwrap();
            reader
                .require_decision_field(&account, AccountStatus::reference())
                .unwrap();
        })
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap();
    reads
        .complete_projected_dependencies()
        .expect("completion must re-observe only the explicitly sealed status dependency");
}

fn authenticated(
    world: &super::super::fixture::AuthorizationWorld,
    subject: &str,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64> {
    let external = world.authenticate(subject, Duration::from_secs(60), request);
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

fn resolved_principal(
    world: &super::super::fixture::AuthorizationWorld,
    identity: u64,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Principal> {
    world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            PrincipalIdentityField::reference(),
            identity,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}

fn link_program(
    world: &super::super::fixture::AuthorizationWorld,
    actor: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    principal: &WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Principal>,
    account: &WorthQueryApplicationEntityIdentity<
        IdentityExecutionSchema,
        super::super::fixture::Account,
    >,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    account_status: &str,
    relation_key: &str,
) -> WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    ChangeOwnershipOperation,
    ChangeOwnershipInput,
    Principal,
> {
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ChangeOwnershipOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(actor, principal, &operation, Default::default(), request)
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, principal| {
            reader
                .decision_relations_from(AccountOwner::reference(), principal)
                .unwrap();
            let account = reader
                .resolve_entity(AccountStatus::reference(), account_status.to_string())
                .unwrap();
            reader
                .require_decision_field(&account, AccountStatus::reference())
                .unwrap();
        })
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies()
        .unwrap()
        .begin_effect_program();
    let principal = effects.existing_entity(principal).unwrap();
    let account = effects.existing_entity(account).unwrap();
    effects
        .link(
            AccountOwner::reference(),
            relation_key,
            &principal,
            &account,
        )
        .unwrap();
    effects.finish().unwrap()
}

fn assert_membership_absent(
    world: &super::super::fixture::AuthorizationWorld,
    actor: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    principal: &WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Principal>,
    account: &WorthQueryApplicationEntityIdentity<
        IdentityExecutionSchema,
        super::super::fixture::Account,
    >,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) {
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ChangeOwnershipOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(actor, principal, &operation, Default::default(), request)
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, principal| {
            reader
                .decision_relations_from(AccountOwner::reference(), principal)
                .unwrap();
        })
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap()
        .complete_projected_dependencies()
        .unwrap();
    let Err(denial) = reads.projected_relation(AccountOwner::reference(), principal, account)
    else {
        panic!("the committed unlink must remove the exact membership");
    };
    assert_eq!(
        denial.kind(),
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact
    );
}

fn unlink_program(
    world: &super::super::fixture::AuthorizationWorld,
    actor: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    principal: &WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Principal>,
    account: &WorthQueryApplicationEntityIdentity<
        IdentityExecutionSchema,
        super::super::fixture::Account,
    >,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    ChangeOwnershipOperation,
    ChangeOwnershipInput,
    Principal,
> {
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ChangeOwnershipOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(actor, principal, &operation, Default::default(), request)
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, principal| {
            reader
                .decision_relations_from(AccountOwner::reference(), principal)
                .unwrap();
        })
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap()
        .complete_projected_dependencies()
        .unwrap();
    let observed = reads
        .projected_relation(AccountOwner::reference(), principal, account)
        .unwrap();
    let mut effects = reads.begin_effect_program();
    effects.unlink(AccountOwner::reference(), observed).unwrap();
    effects.finish().unwrap()
}
