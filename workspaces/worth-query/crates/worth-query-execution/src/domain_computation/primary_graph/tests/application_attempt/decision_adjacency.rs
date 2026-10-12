use std::time::Duration;
use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;

#[path = "decision_adjacency/duplicate_retirement.rs"]
mod duplicate_retirement;
#[path = "decision_adjacency/product_currentness.rs"]
mod product_currentness;

use super::super::fixture::{
    AccountOwner, AccountStatus, ChangeOwnershipInput, ChangeOwnershipOperation,
    IdentityExecutionSchema, Principal, PrincipalIdentityField, TouchAccountOperation,
};
use super::{idempotency, installed_authorization_world, live_scope, resolved_account};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationEntityIdentity, WorthQueryAuthenticatedPrincipal,
    WorthQueryInvariantProjectionTraversalDenialKind, WorthQueryPrincipalResolutionMode,
};

#[path = "decision_adjacency/an_edge_entering_an_observed_empty_adjacency_stales_the_attempt.rs"]
mod an_edge_entering_an_observed_empty_adjacency_stales_the_attempt;

#[path = "decision_adjacency/sealed_adjacency_membership_supplies_exact_unlink_evidence.rs"]
mod sealed_adjacency_membership_supplies_exact_unlink_evidence;

#[path = "decision_adjacency/removing_an_observed_present_relation_stales_a_competing_program.rs"]
mod removing_an_observed_present_relation_stales_a_competing_program;

fn assert_changed_decision(outcome: WorthQueryApplicationCommitOutcome, cause: &str) {
    let WorthQueryApplicationCommitOutcome::Stale(stale) = outcome else {
        panic!("{cause} must stale before effects: {outcome:?}");
    };
    assert!(stale.stale_fact_count() > 0);
}

#[path = "decision_adjacency/capability_relation_traversal_does_not_enter_sealed_decision_dep.rs"]
mod capability_relation_traversal_does_not_enter_sealed_decision_dep;

fn authenticated(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &super::super::fixture::AuthorizationWorld,
    subject: &str,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64> {
    let external = world.authenticate(subject, Duration::from_secs(60), request);
    world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            phase_custody,
            &world.binding,
            &external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}

fn resolved_principal(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &super::super::fixture::AuthorizationWorld,
    identity: u64,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Principal> {
    world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            phase_custody,
            PrincipalIdentityField::reference(),
            identity,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}

fn link_program(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
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
        .selected_product(&(phase_custody).test_host_request())
        .authorize_operation(
            phase_custody,
            actor,
            principal,
            &operation,
            Default::default(),
            request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            phase_custody,
            &admission,
            |reader, principal| {
                reader
                    .decision_relations_from(AccountOwner::reference(), principal)
                    .unwrap();
                let account = reader
                    .resolve_entity(AccountStatus::reference(), account_status.to_string())
                    .unwrap();
                reader
                    .require_decision_field(&account, AccountStatus::reference())
                    .unwrap();
            },
            AllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(
            phase_custody,
            admission,
            projection,
            AllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies(
            phase_custody,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
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
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &super::super::fixture::AuthorizationWorld,
    actor: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    principal: &WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Principal>,
    account: &WorthQueryApplicationEntityIdentity<
        IdentityExecutionSchema,
        super::super::fixture::Account,
    >,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    account_status: &str,
) {
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ChangeOwnershipOperation::reference())
        .unwrap();
    let admission = world
        .selected_product(&(phase_custody).test_host_request())
        .authorize_operation(
            phase_custody,
            actor,
            principal,
            &operation,
            Default::default(),
            request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            phase_custody,
            &admission,
            |reader, principal| {
                reader
                    .decision_relations_from(AccountOwner::reference(), principal)
                    .unwrap();
                let account = reader
                    .resolve_entity(AccountStatus::reference(), account_status.to_owned())
                    .unwrap();
                reader
                    .require_decision_field(&account, AccountStatus::reference())
                    .unwrap();
            },
            AllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(
            phase_custody,
            admission,
            projection,
            AllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .complete_projected_dependencies(
            phase_custody,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let Err(denial) = reads.projected_relation(AccountOwner::reference(), principal, account)
    else {
        panic!("the committed unlink must remove the exact membership");
    };
    assert_eq!(
        denial.kind(),
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact
    );
    let mut candidate = reads.begin_effect_program();
    let from = candidate.existing_entity(principal).unwrap();
    let to = candidate.existing_entity(account).unwrap();
    let denial = candidate
        .unlink_observed(AccountOwner::reference(), &from, &to)
        .expect_err("candidate authoring cannot reconstruct an unobserved edge");
    assert_eq!(
        denial.kind(),
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact
    );
}

fn unlink_program(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
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
        .selected_product(&(phase_custody).test_host_request())
        .authorize_operation(
            phase_custody,
            actor,
            principal,
            &operation,
            Default::default(),
            request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            phase_custody,
            &admission,
            |reader, principal| {
                reader
                    .decision_relations_from(AccountOwner::reference(), principal)
                    .unwrap();
            },
            AllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(
            phase_custody,
            admission,
            projection,
            AllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .complete_projected_dependencies(
            phase_custody,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let mut effects = reads.begin_effect_program();
    let from = effects.existing_entity(principal).unwrap();
    let to = effects.existing_entity(account).unwrap();
    effects
        .unlink_observed(AccountOwner::reference(), &from, &to)
        .unwrap();
    effects.finish().unwrap()
}

#[path = "decision_adjacency/admitted_capability_manifest.rs"]
mod admitted_capability_manifest;
