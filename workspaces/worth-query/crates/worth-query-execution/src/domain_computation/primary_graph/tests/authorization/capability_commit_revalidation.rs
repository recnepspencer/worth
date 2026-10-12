use std::collections::BTreeMap;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, StringApplicationValueBinding,
};
use worth_query_declaration::facade::authentication::{
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

use super::super::application_attempt::{authenticated_principal, idempotency};
use super::super::fixture::{
    installed_capability_authorization_world, installed_capability_replacement_world, live_scope,
    publish_relational_mutation, AccountLabel, AccountStatus, CapabilityIdentity, CapabilityStatus,
    CapabilityStatusBinding, CapabilityStatusField, CapabilityTouchOperation,
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

#[path = "capability_commit_revalidation/revoked_access_cannot_progress_to_operation_authority.rs"]
mod revoked_access_cannot_progress_to_operation_authority;

#[test]
fn equivalent_retry_denies_before_receipt_after_grant_revocation() {
    let world = current_retry_world();
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let (first, retry) = world
        .application
        .with_host_advancement(|phase| equivalent_programs(&phase, &world, &principal, &request))
        .expect("the fixture read owns its installed host call");
    commit_first(&world, first, 51);
    world
        .application
        .with_host_advancement(|phase| revoke_grant(&phase, &world))
        .expect("the fixture read owns its installed host call");

    assert_readmission_denial(world.application.compare_and_commit_application(
        retry,
        idempotency(51, 51),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    ));
}

#[path = "capability_commit_revalidation/equivalent_retry_denies_before_receipt_after_principal_disableme.rs"]
mod equivalent_retry_denies_before_receipt_after_principal_disableme;

#[test]
fn equivalent_retry_denies_before_receipt_after_capability_expiry() {
    let world = current_retry_world();
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let (first, retry) = world
        .application
        .with_host_advancement(|phase| equivalent_programs(&phase, &world, &principal, &request))
        .expect("the fixture read owns its installed host call");
    commit_first(&world, first, 53);
    world.authorization_time.script([time(300)]);

    assert_readmission_denial(world.application.compare_and_commit_application(
        retry,
        idempotency(53, 53),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    ));
}

#[path = "capability_commit_revalidation/future_equivalent_grant_cannot_inherit_an_expired_access_context.rs"]
mod future_equivalent_grant_cannot_inherit_an_expired_access_context;

#[path = "capability_commit_revalidation/idempotency_inspection_denies_before_receipt_after_grant_revocat.rs"]
mod idempotency_inspection_denies_before_receipt_after_grant_revocat;

#[test]
fn equivalent_retry_denies_before_receipt_after_resource_workflow_drift() {
    let world = current_retry_world();
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let (first, retry) = world
        .application
        .with_host_advancement(|phase| equivalent_programs(&phase, &world, &principal, &request))
        .expect("the fixture read owns its installed host call");
    commit_first(&world, first, 54);
    world
        .application
        .with_host_advancement(|phase| change_account_status(&phase, &world, "account-1", "closed"))
        .expect("the fixture read owns its installed host call");

    assert_readmission_denial(world.application.compare_and_commit_application(
        retry,
        idempotency(54, 54),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    ));
}

#[path = "capability_commit_revalidation/unrelated_graph_drift_preserves_current_idempotent_recovery.rs"]
mod unrelated_graph_drift_preserves_current_idempotent_recovery;

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
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &World,
    principal: &crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal<
        super::super::fixture::IdentityExecutionSchema,
        super::super::fixture::Principal,
        u64,
    >,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> (Program, Program) {
    let first =
        admitted_capability_program(phase_custody, world, principal, request, "committed").0;
    let retry =
        admitted_capability_program(phase_custody, world, principal, request, "committed").0;
    (first, retry)
}

fn commit_first(world: &World, program: Program, key: u8) -> WorthQueryApplicationCommitReceipt {
    let WorthQueryApplicationCommitOutcome::Committed(receipt) =
        world.application.compare_and_commit_application(
            program,
            idempotency(key, key),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
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

pub(super) fn revoke_grant(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &World,
) {
    let request = live_scope();
    let grant = world
        .selected_product(&(phase_custody).test_host_request())
        .resolve_entity(
            phase_custody,
            CapabilityIdentity::reference(),
            "capability-1".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let field = CapabilityStatusField::reference();
    let locator = installed_field(world, field.entity(), field.aspect(), field.field());
    update_field(
        &phase_custody.borrowed_request_for_test(),
        world,
        grant.entity_id(),
        locator,
        CapabilityStatusBinding::encode(&CapabilityStatus::Revoked).unwrap(),
        "revoke-capability-grant",
    );
}

pub(super) fn disable_mapping(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &World,
    mapping: EntityId,
) {
    let graph = world.application.runtime.primary_graph().unwrap();
    let layout = graph
        .layout()
        .principal_binding(world.binding.binding())
        .unwrap();
    update_field(
        execution,
        world,
        mapping,
        layout.status_locator.clone(),
        WorthQueryPrincipalMappingStatusBinding::encode(
            &WorthQueryPrincipalMappingStatus::Disabled,
        )
        .unwrap(),
        "disable-capability-principal",
    );
}

pub(super) fn enable_mapping(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &World,
    mapping: EntityId,
) {
    let graph = world.application.runtime.primary_graph().unwrap();
    let layout = graph
        .layout()
        .principal_binding(world.binding.binding())
        .unwrap();
    update_field(
        execution,
        world,
        mapping,
        layout.status_locator.clone(),
        WorthQueryPrincipalMappingStatusBinding::encode(&WorthQueryPrincipalMappingStatus::Enabled)
            .unwrap(),
        "reenable-capability-principal",
    );
}

fn change_account_status(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &World,
    key: &str,
    status: &str,
) {
    let field = AccountStatus::reference();
    change_account_field(
        phase_custody,
        world,
        key,
        (field.entity(), field.aspect(), field.field()),
        StringApplicationValueBinding::encode(&status.to_owned()).unwrap(),
        "change-capability-resource-workflow",
    );
}

fn change_account_label(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &World,
    key: &str,
    label: &str,
) {
    let field = AccountLabel::reference();
    change_account_field(
        phase_custody,
        world,
        key,
        (field.entity(), field.aspect(), field.field()),
        StringApplicationValueBinding::encode(&label.to_owned()).unwrap(),
        "change-unrelated-capability-fact",
    );
}

fn change_account_field(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &World,
    key: &str,
    field: (&str, &str, &str),
    value: AspectValue,
    reason: &str,
) {
    let request = live_scope();
    let account = world
        .selected_product(&(phase_custody).test_host_request())
        .resolve_entity(
            phase_custody,
            super::super::fixture::AccountIdentity::reference(),
            key.to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let locator = installed_field(world, field.0, field.1, field.2);
    update_field(
        &phase_custody.borrowed_request_for_test(),
        world,
        account.entity_id(),
        locator,
        value,
        reason,
    );
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &World,
    entity_id: EntityId,
    locator: AspectFieldLocator,
    value: AspectValue,
    reason: &str,
) {
    let fields = AspectFieldPatch::from(BTreeMap::from([(locator, value)]));
    publish_relational_mutation(
        execution,
        world,
        WorkerIntentBatch::new(reason).push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent { entity_id, fields }),
        )),
    );
}
