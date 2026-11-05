use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::LiveActivityEffect;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::live_account_query::LiveActivityEvent;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
use worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub fn commit_live_activity(
    world: &AuthorizationWorld,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &WorthQueryRequestScope,
) -> WorthQueryApplicationCommitReceipt {
    commit_live_activity_with_label(world, principal, request, "live-delivered")
}

pub fn commit_live_activity_with_label(
    world: &AuthorizationWorld,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &WorthQueryRequestScope,
    label: &str,
) -> WorthQueryApplicationCommitReceipt {
    commit_live_activity_with_identity(world, principal, request, label, 227, 91)
}

pub fn commit_live_activity_with_identity(
    world: &AuthorizationWorld,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &WorthQueryRequestScope,
    label: &str,
    idempotency_key: u8,
    request_digest: u8,
) -> WorthQueryApplicationCommitReceipt {
    commit_live_activity_on_product(
        world,
        world
            .application
            .product_runtime()
            .default_branch_for_test(),
        principal,
        request,
        label,
        idempotency_key,
        request_digest,
    )
}

pub fn commit_live_activity_on_product(
    world: &AuthorizationWorld,
    product: &worth_runtime_world::facade::ProductBranchIdentity,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &WorthQueryRequestScope,
    label: &str,
    idempotency_key: u8,
    request_digest: u8,
) -> WorthQueryApplicationCommitReceipt {
    let program = world
        .application
        .with_host_advancement(|phase| {
            let phase_custody = phase.test_host_request_for_test();
            let request_execution = phase
                .execution_request_for_for_test(world.application.product_runtime())
                .expect("the fixture lends its installed phase");
            let execution = &request_execution;
            let account = world
                .application
                .select_product_branch_for_test(execution, product)
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase_custody.phase_for_application_for_test(&world.application),
                    AccountStatus::reference(),
                    "open".to_owned(),
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            live_activity_program(
                &phase_custody,
                &(phase_custody).borrowed_request_for_test(),
                world,
                product,
                principal,
                &account,
                request,
                label,
            )
        })
        .expect("the live fixture opens its preparation call");
    match world.application.compare_and_commit_application(
        program,
        WorthQueryApplicationIdempotencyBinding::new_for_test(
            [idempotency_key; 32],
            [request_digest; 32],
        ),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    ) {
        WorthQueryApplicationCommitOutcome::Committed(receipt) => receipt,
        unexpected => panic!("live activity fixture must commit: {unexpected:?}"),
    }
}

pub fn live_activity_program(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    product: &worth_runtime_world::facade::ProductBranchIdentity,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    account: &WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>,
    request: &WorthQueryRequestScope,
    label: &str,
) -> worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    TouchAccountOperation,
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::TouchAccountInput,
    Account,
>{
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .application
        .select_product_branch_for_test(execution, product)
        .expect("the selected live product remains admitted")
        .authorize_operation(
            &phase_custody.phase_for_application_for_test(&world.application),
            principal,
            account,
            &operation,
            Default::default(),
            request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            &phase_custody.phase_for_application_for_test(&world.application),
            &admission,
            |reader, projected| {
                reader
                    .require_decision_field(projected, AccountStatus::reference())
                    .unwrap();
                reader
                    .require_decision_field(projected, AccountLabel::reference())
                    .unwrap();
            },
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(
            &phase_custody.phase_for_application_for_test(&world.application),
            admission,
            projection,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies(
            &phase_custody.phase_for_application_for_test(&world.application),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .begin_effect_program();
    let account = effects.existing_entity(account).unwrap();
    effects
        .write_field(&account, AccountLabel::reference(), label.to_owned())
        .unwrap();
    effects
        .emit(
            LiveActivityEffect::reference(),
            LiveActivityEvent::new("account-1", "activity-primary"),
        )
        .unwrap();
    effects.finish().unwrap()
}
