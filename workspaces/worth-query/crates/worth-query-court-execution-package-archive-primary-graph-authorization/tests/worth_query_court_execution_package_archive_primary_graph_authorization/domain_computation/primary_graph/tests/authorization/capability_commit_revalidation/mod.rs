pub use std::collections::BTreeMap;

pub use worth_foundational::facade::AspectFieldLocator;
pub use worth_foundational::facade::AspectValue;
pub use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
pub use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
pub use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
pub use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatusBinding;
pub use worth_relational::facade::identity::EntityId;
pub use worth_relational::facade::transactions::AspectFieldPatch;
pub use worth_relational::facade::transactions::EntityMutationIntent;
pub use worth_relational::facade::transactions::MutationIntent;
pub use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
pub use worth_relational::facade::transactions::WorkerIntentBatch;

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_capability_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_capability_replacement_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityIdentity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityStatus;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityStatusBinding;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityStatusField;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityTouchOperation;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_progression::admitted_capability_access;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_progression::admitted_capability_operation;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_progression::admitted_capability_program;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_progression::time;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_progression::Program;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitTerminalKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyResolutionDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub type World = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;

#[test]
fn equivalent_retry_denies_before_receipt_after_grant_revocation() {
    let world = current_retry_world();
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
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
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    ));
}

#[test]
fn equivalent_retry_denies_before_receipt_after_capability_expiry() {
    let world = current_retry_world();
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
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
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    ));
}

#[test]
fn equivalent_retry_denies_before_receipt_after_resource_workflow_drift() {
    let world = current_retry_world();
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
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
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    ));
}

pub fn current_retry_world() -> World {
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

pub fn equivalent_programs(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &World,
    principal: &worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal<
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal,
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

pub fn commit_first(
    world: &World,
    program: Program,
    key: u8,
) -> WorthQueryApplicationCommitReceipt {
    let WorthQueryApplicationCommitOutcome::Committed(receipt) =
        world.application.compare_and_commit_application(
            program,
            idempotency(key, key),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the current first application attempt must commit");
    };
    receipt
}

pub fn assert_readmission_denial(outcome: WorthQueryApplicationCommitOutcome) {
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

pub fn revoke_grant(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &World,
) {
    let request = live_scope();
    let grant = world
        .selected_product(&(phase_custody).test_host_request_for_test())
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
        grant.entity_id_for_test(),
        locator,
        CapabilityStatusBinding::encode(&CapabilityStatus::Revoked).unwrap(),
        "revoke-capability-grant",
    );
}

pub fn disable_mapping(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &World,
    mapping: EntityId,
) {
    let graph = (*world.application.runtime_for_test()).primary_graph().unwrap();
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

pub fn enable_mapping(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &World,
    mapping: EntityId,
) {
    let graph = (*world.application.runtime_for_test()).primary_graph().unwrap();
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

pub fn change_account_status(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
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

pub fn change_account_label(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
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

pub fn change_account_field(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &World,
    key: &str,
    field: (&str, &str, &str),
    value: AspectValue,
    reason: &str,
) {
    let request = live_scope();
    let account = world
        .selected_product(&(phase_custody).test_host_request_for_test())
        .resolve_entity(
            phase_custody,
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity::reference(),
            key.to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let locator = installed_field(world, field.0, field.1, field.2);
    update_field(
        &phase_custody.borrowed_request_for_test(),
        world,
        account.entity_id_for_test(),
        locator,
        value,
        reason,
    );
}

pub fn installed_field(
    world: &World,
    entity: &str,
    aspect: &str,
    field: &str,
) -> AspectFieldLocator {
    (*world
        .application
        .runtime_for_test())
        .primary_graph()
        .unwrap()
        .layout()
        .field_locator(entity, aspect, field)
        .unwrap()
        .clone()
}

pub fn update_field(
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

#[path = "equivalent_retry_denies_before_receipt_after_principal_disableme/mod.rs"]
pub mod equivalent_retry_denies_before_receipt_after_principal_disableme;
#[path = "future_equivalent_grant_cannot_inherit_an_expired_access_context/mod.rs"]
pub mod future_equivalent_grant_cannot_inherit_an_expired_access_context;
#[path = "idempotency_inspection_denies_before_receipt_after_grant_revocat/mod.rs"]
pub mod idempotency_inspection_denies_before_receipt_after_grant_revocat;
#[path = "revoked_access_cannot_progress_to_operation_authority/mod.rs"]
pub mod revoked_access_cannot_progress_to_operation_authority;
#[path = "unrelated_graph_drift_preserves_current_idempotent_recovery/mod.rs"]
pub mod unrelated_graph_drift_preserves_current_idempotent_recovery;
