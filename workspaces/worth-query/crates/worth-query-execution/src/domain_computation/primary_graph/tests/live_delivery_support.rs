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
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let program = live_activity_program(world, principal, &account, request, label);
    match world.application.compare_and_commit_application(
        program,
        WorthQueryApplicationIdempotencyBinding::new([227; 32], [91; 32]),
    ) {
        WorthQueryApplicationCommitOutcome::Committed(receipt) => receipt,
        unexpected => panic!("live activity fixture must commit: {unexpected:?}"),
    }
}

fn live_activity_program(
    world: &AuthorizationWorld,
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
        .selected_product()
        .authorize_operation(principal, account, &operation, Default::default(), request)
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, projected| {
            reader
                .require_decision_field(projected, AccountStatus::reference())
                .unwrap();
            reader
                .require_decision_field(projected, AccountLabel::reference())
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
