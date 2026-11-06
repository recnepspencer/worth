pub use std::time::Duration;
pub use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountOwner;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::ChangeOwnershipInput;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ChangeOwnershipOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::PrincipalIdentityField;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityIdentity;
pub use worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal;
pub use worth_query_execution::facade::domain_computation::WorthQueryInvariantProjectionTraversalDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub fn assert_changed_decision(outcome: WorthQueryApplicationCommitOutcome, cause: &str) {
    let WorthQueryApplicationCommitOutcome::Stale(stale) = outcome else {
        panic!("{cause} must stale before effects: {outcome:?}");
    };
    assert!(stale.stale_fact_count() > 0);
}

pub fn authenticated(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    subject: &str,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64> {
    let external = world.authenticate(subject, Duration::from_secs(60), request);
    world
        .application
        .select_product_branch_for_test(
            execution,
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
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

pub fn resolved_principal(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    identity: u64,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Principal> {
    world
        .application
        .select_product_branch_for_test(
            execution,
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            phase_custody,
            PrincipalIdentityField::at("PrincipalIdentityField"),
            identity,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}

pub fn link_program(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    actor: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    principal: &WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Principal>,
    account: &WorthQueryApplicationEntityIdentity<
        IdentityExecutionSchema,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account,
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
        .selected_product(&(phase_custody).test_host_request_for_test())
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
                    .resolve_entity(
                        AccountStatus::at("AccountStatus"),
                        account_status.to_string(),
                    )
                    .unwrap();
                reader
                    .require_decision_field(&account, AccountStatus::at("AccountStatus"))
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
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
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

pub fn assert_membership_absent(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    actor: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    principal: &WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Principal>,
    account: &WorthQueryApplicationEntityIdentity<
        IdentityExecutionSchema,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account,
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
        .selected_product(&(phase_custody).test_host_request_for_test())
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
                    .resolve_entity(
                        AccountStatus::at("AccountStatus"),
                        account_status.to_owned(),
                    )
                    .unwrap();
                reader
                    .require_decision_field(&account, AccountStatus::at("AccountStatus"))
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
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let Err(denial) = reads.projected_relation(AccountOwner::reference(), principal, account)
    else {
        panic!("the committed unlink must remove the exact membership");
    };
    assert_eq!(
        denial.kind(),
        worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact
    );
    let mut candidate = reads.begin_effect_program();
    let from = candidate.existing_entity(principal).unwrap();
    let to = candidate.existing_entity(account).unwrap();
    let denial = candidate
        .unlink_observed_for_test(AccountOwner::reference(), &from, &to)
        .expect_err("candidate authoring cannot reconstruct an unobserved edge");
    assert_eq!(
        denial.kind(),
        worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact
    );
}

pub fn unlink_program(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    actor: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    principal: &WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Principal>,
    account: &WorthQueryApplicationEntityIdentity<
        IdentityExecutionSchema,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account,
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
        .selected_product(&(phase_custody).test_host_request_for_test())
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
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let mut effects = reads.begin_effect_program();
    let from = effects.existing_entity(principal).unwrap();
    let to = effects.existing_entity(account).unwrap();
    effects
        .unlink_observed_for_test(AccountOwner::reference(), &from, &to)
        .unwrap();
    effects.finish().unwrap()
}

#[path = "admitted_capability_manifest/mod.rs"]
pub mod admitted_capability_manifest;
#[path = "an_edge_entering_an_observed_empty_adjacency_stales_the_attempt/mod.rs"]
pub mod an_edge_entering_an_observed_empty_adjacency_stales_the_attempt;
#[path = "capability_relation_traversal_does_not_enter_sealed_decision_dep/mod.rs"]
pub mod capability_relation_traversal_does_not_enter_sealed_decision_dep;
#[path = "duplicate_retirement/mod.rs"]
pub mod duplicate_retirement;
#[path = "product_currentness/mod.rs"]
pub mod product_currentness;
#[path = "removing_an_observed_present_relation_stales_a_competing_program/mod.rs"]
pub mod removing_an_observed_present_relation_stales_a_competing_program;
#[path = "sealed_adjacency_membership_supplies_exact_unlink_evidence/mod.rs"]
pub mod sealed_adjacency_membership_supplies_exact_unlink_evidence;
