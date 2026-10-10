use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

use super::fixture::{
    Account, AccountLabel, AccountStatus, AuthorizationWorld, IdentityExecutionSchema,
    LiveActivityEffect, LiveActivityEvent, Principal, TouchAccountOperation,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationEntityIdentity, WorthQueryApplicationIdempotencyBinding,
    WorthQueryAuthenticatedPrincipal, WorthQueryPrincipalResolutionMode,
};

pub(in crate::domain_computation::primary_graph) fn commit_live_activity(
    world: &AuthorizationWorld,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &WorthQueryRequestScope,
) -> WorthQueryApplicationCommitReceipt {
    commit_live_activity_with_label(world, principal, request, "live-delivered")
}

pub(in crate::domain_computation::primary_graph) fn commit_live_activity_with_label(
    world: &AuthorizationWorld,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &WorthQueryRequestScope,
    label: &str,
) -> WorthQueryApplicationCommitReceipt {
    commit_live_activity_with_identity(world, principal, request, label, 227, 91)
}

pub(in crate::domain_computation::primary_graph) fn commit_live_activity_with_identity(
    world: &AuthorizationWorld,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &WorthQueryRequestScope,
    label: &str,
    idempotency_key: u8,
    request_digest: u8,
) -> WorthQueryApplicationCommitReceipt {
    commit_live_activity_on_product(
        world,
        world.application.product_runtime().default_branch(),
        principal,
        request,
        label,
        idempotency_key,
        request_digest,
    )
}

pub(in crate::domain_computation::primary_graph) fn commit_live_activity_on_product(
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
            let phase_custody = phase.test_host_request();
            let request_execution = phase
                .execution_request_for(world.application.product_runtime())
                .expect("the fixture lends its installed phase");
            let execution = &request_execution;
            let account = world
                .application
                .select_product_branch(execution, product)
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase_custody.phase_for_application(&world.application),
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
        WorthQueryApplicationIdempotencyBinding::new([idempotency_key; 32], [request_digest; 32]),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    ) {
        WorthQueryApplicationCommitOutcome::Committed(receipt) => receipt,
        unexpected => panic!("live activity fixture must commit: {unexpected:?}"),
    }
}

pub(in crate::domain_computation::primary_graph) fn live_activity_program(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    product: &worth_runtime_world::facade::ProductBranchIdentity,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    account: &WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>,
    request: &WorthQueryRequestScope,
    label: &str,
) -> crate::domain_computation::primary_graph::WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    TouchAccountOperation,
    super::fixture::TouchAccountInput,
    Account,
> {
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .application
        .select_product_branch(execution, product)
        .expect("the selected live product remains admitted")
        .authorize_operation(
            &phase_custody.phase_for_application(&world.application),
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
            &phase_custody.phase_for_application(&world.application),
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
            &phase_custody.phase_for_application(&world.application),
            admission,
            projection,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies(
            &phase_custody.phase_for_application(&world.application),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
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
