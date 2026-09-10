use std::collections::BTreeMap;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_query_installation::facade::TypedApplicationValue;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

use super::super::application_attempt::{authenticated_principal, idempotency};
use super::super::fixture::{
    installed_capability_authorization_world, installed_capability_replacement_world, live_scope,
    publish_relational_mutation, AccountLabel, AccountStatus, CapabilityIdentity, CapabilityStatus,
    CapabilityStatusField, CapabilityTouchOperation,
};
use super::capability_progression::{
    admitted_capability_access, admitted_capability_operation, admitted_capability_program, time,
    Program,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationCommitTerminalKind, WorthQueryApplicationIdempotencyResolutionDenialKind,
    WorthQueryOperationAuthorizationDenialKind, WorthQueryPrincipalResolutionMode,
};

type World = super::super::fixture::AuthorizationWorld;

#[test]
fn revoked_access_cannot_progress_to_operation_authority() {
    let world = installed_capability_authorization_world();
    world.authorization_time.script([time(100), time(100)]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let access = admitted_capability_access(&world, &principal, &request, 100).unwrap();
    revoke_grant(&world);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(CapabilityTouchOperation::reference())
        .unwrap();

    let Err(denial) =
        world
            .application
            .authorize_capability_operation(access, &operation, Default::default())
    else {
        panic!("revoked retained access cannot become operation authority");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
}

#[test]
fn equivalent_retry_denies_before_receipt_after_grant_revocation() {
    let world = current_retry_world();
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let (first, retry) = equivalent_programs(&world, &principal, &request);
    commit_first(&world, first, 51);
    revoke_grant(&world);

    assert_readmission_denial(
        world
            .application
            .compare_and_commit_application(retry, idempotency(51, 51)),
    );
}

#[test]
fn equivalent_retry_denies_before_receipt_after_principal_disablement() {
    let world = current_retry_world();
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let (first, retry) = equivalent_programs(&world, &principal, &request);
    commit_first(&world, first, 52);
    disable_mapping(&world, principal.mapping_entity_id());

    assert_readmission_denial(
        world
            .application
            .compare_and_commit_application(retry, idempotency(52, 52)),
    );
}

#[test]
fn equivalent_retry_denies_before_receipt_after_capability_expiry() {
    let world = current_retry_world();
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let (first, retry) = equivalent_programs(&world, &principal, &request);
    commit_first(&world, first, 53);
    world.authorization_time.script([time(300)]);

    assert_readmission_denial(
        world
            .application
            .compare_and_commit_application(retry, idempotency(53, 53)),
    );
}

#[test]
fn future_equivalent_grant_cannot_inherit_an_expired_access_context() {
    let world = installed_capability_replacement_world();
    world.authorization_time.script([
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
    ]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let (first, retry) = equivalent_programs(&world, &principal, &request);
    commit_first(&world, first, 56);
    world.authorization_time.script([time(120)]);

    assert_readmission_denial(
        world
            .application
            .compare_and_commit_application(retry, idempotency(56, 56)),
    );
}

#[test]
fn idempotency_inspection_denies_before_receipt_after_grant_revocation() {
    let world = current_retry_world();
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let first = admitted_capability_program(&world, &principal, &request, "committed").0;
    let admission = admitted_capability_operation(&world, &principal, &request);
    commit_first(&world, first, 57);
    revoke_grant(&world);

    let Err(denial) = world
        .application
        .resolve_admitted_application_idempotency(&admission, idempotency(57, 57))
    else {
        panic!("stale capability facts cannot inspect a committed idempotency receipt");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationIdempotencyResolutionDenialKind::Authorization
    );
    assert_eq!(
        denial.authorization().map(|denial| denial.kind()),
        Some(WorthQueryOperationAuthorizationDenialKind::StaleAuthorization)
    );
}

#[test]
fn equivalent_retry_denies_before_receipt_after_resource_workflow_drift() {
    let world = current_retry_world();
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let (first, retry) = equivalent_programs(&world, &principal, &request);
    commit_first(&world, first, 54);
    change_account_status(&world, "account-1", "closed");

    assert_readmission_denial(
        world
            .application
            .compare_and_commit_application(retry, idempotency(54, 54)),
    );
}

#[test]
fn unrelated_graph_drift_preserves_current_idempotent_recovery() {
    let world = current_retry_world();
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let (first, retry) = equivalent_programs(&world, &principal, &request);
    let committed = commit_first(&world, first, 55);
    change_account_label(&world, "account-2", "independently-updated");

    let outcome = world
        .application
        .compare_and_commit_application(retry, idempotency(55, 55));
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) = outcome else {
        panic!("unrelated drift must preserve lawful idempotent recovery: {outcome:?}");
    };
    assert!(recovered.is_same_authoritative_commit(&committed));
    assert_eq!(
        recovered.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Recovered
    );
}

fn current_retry_world() -> World {
    let world = installed_capability_authorization_world();
    world.authorization_time.script([
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
    ]);
    world
}

fn equivalent_programs(
    world: &World,
    principal: &crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal<
        super::super::fixture::IdentityExecutionSchema,
        super::super::fixture::Principal,
        u64,
    >,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> (Program, Program) {
    let first = admitted_capability_program(world, principal, request, "committed").0;
    let retry = admitted_capability_program(world, principal, request, "committed").0;
    (first, retry)
}

fn commit_first(world: &World, program: Program, key: u8) -> WorthQueryApplicationCommitReceipt {
    let WorthQueryApplicationCommitOutcome::Committed(receipt) = world
        .application
        .compare_and_commit_application(program, idempotency(key, key))
    else {
        panic!("the current first application attempt must commit");
    };
    receipt
}

fn assert_readmission_denial(outcome: WorthQueryApplicationCommitOutcome) {
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("authorization drift must deny before disclosing an idempotency receipt");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProviderRejected
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::DecisionReadSet
    );
}

fn revoke_grant(world: &World) {
    let request = live_scope();
    let grant = world
        .selected_product()
        .resolve_entity(
            CapabilityIdentity::reference(),
            "capability-1".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let field = CapabilityStatusField::reference();
    let locator = installed_field(world, field.entity(), field.aspect(), field.field());
    update_field(
        world,
        grant.entity_id(),
        locator,
        CapabilityStatus::Revoked.into_foundational_value(),
        "revoke-capability-grant",
    );
}

fn disable_mapping(world: &World, mapping: EntityId) {
    let graph = world.application.runtime.primary_graph().unwrap();
    let layout = graph
        .layout()
        .principal_binding(world.binding.binding())
        .unwrap();
    update_field(
        world,
        mapping,
        layout.status_locator.clone(),
        WorthQueryPrincipalMappingStatus::Disabled.into_foundational_value(),
        "disable-capability-principal",
    );
}

fn change_account_status(world: &World, key: &str, status: &str) {
    let field = AccountStatus::reference();
    change_account_field(
        world,
        key,
        (field.entity(), field.aspect(), field.field()),
        status.to_owned().into_foundational_value(),
        "change-capability-resource-workflow",
    );
}

fn change_account_label(world: &World, key: &str, label: &str) {
    let field = AccountLabel::reference();
    change_account_field(
        world,
        key,
        (field.entity(), field.aspect(), field.field()),
        label.to_owned().into_foundational_value(),
        "change-unrelated-capability-fact",
    );
}

fn change_account_field(
    world: &World,
    key: &str,
    field: (&str, &str, &str),
    value: AspectValue,
    reason: &str,
) {
    let request = live_scope();
    let account = world
        .selected_product()
        .resolve_entity(
            super::super::fixture::AccountIdentity::reference(),
            key.to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let locator = installed_field(world, field.0, field.1, field.2);
    update_field(world, account.entity_id(), locator, value, reason);
}

fn installed_field(world: &World, entity: &str, aspect: &str, field: &str) -> AspectFieldLocator {
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .layout()
        .field_locator(entity, aspect, field)
        .unwrap()
        .clone()
}

fn update_field(
    world: &World,
    entity_id: EntityId,
    locator: AspectFieldLocator,
    value: AspectValue,
    reason: &str,
) {
    let fields = AspectFieldPatch::from(BTreeMap::from([(locator, value)]));
    publish_relational_mutation(
        world,
        WorkerIntentBatch::new(reason).push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent { entity_id, fields }),
        )),
    );
}
