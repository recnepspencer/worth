use crate::domain_computation::primary_graph::tests::application_attempt::{
    authenticated_principal, idempotency, resolved_account,
};
use crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use crate::domain_computation::primary_graph::tests::fixture::schema_types::{
    live_scope, MultiFieldRetentionInput,
};
use crate::domain_computation::primary_graph::tests::fixture::{
    Account, AccountLabel, AccountStatus, IdentityExecutionSchema, MultiFieldRetentionOperation,
};
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
use worth_query_execution::facade::domain_computation::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationEntityIdentity, WorthQueryRecoveryHandle,
};

type MultiFieldAdmission = WorthQueryAdmittedApplicationOperation<
    IdentityExecutionSchema,
    MultiFieldRetentionOperation,
    MultiFieldRetentionInput,
    Account,
>;

pub fn outboxless_handle(
    world: &AuthorizationWorld,
) -> (WorthQueryRecoveryHandle, MultiFieldAdmission) {
    let outcome = commit_same_record(world, 75, 76);
    let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
        panic!("the two-field inverse fixture must commit: {outcome:?}");
    };
    assert!(receipt.dispatch_outbox().is_none());
    let handle = world
        .application
        .with_host_advancement(|phase| world.application.mint_recovery_handle(&phase, &receipt))
        .unwrap()
        .unwrap();
    let (admission, _) = admit(world, "frozen");
    (handle, admission)
}

fn admit(
    world: &AuthorizationWorld,
    status: &str,
) -> (
    MultiFieldAdmission,
    WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>,
) {
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                world,
                status,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(MultiFieldRetentionOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    TypedMutationPreconditions::new(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    (admission, account)
}

/// Commit a valid same-record change to both fields required by the inverse.
pub fn commit_same_record(
    world: &AuthorizationWorld,
    key: u8,
    intent: u8,
) -> WorthQueryApplicationCommitOutcome {
    let (admission, account) = admit(world, "open");
    let (_, projection, _) = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, projected| {
                        reader
                            .require_decision_field(projected, AccountStatus::at("AccountStatus"))
                            .unwrap();
                        reader
                            .require_decision_field(projected, AccountLabel::at("AccountLabel"))
                            .unwrap();
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("the fixture read owns its installed host call");
    let mut effects = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .complete_projected_dependencies(
                    &phase,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .begin_effect_program()
        })
        .expect("the fixture read owns its installed host call");
    let account = effects.existing_entity(&account).unwrap();
    effects
        .write_field(
            &account,
            AccountStatus::at("AccountStatus"),
            "frozen".to_owned(),
        )
        .unwrap();
    effects
        .write_field(
            &account,
            AccountLabel::at("AccountLabel"),
            "renamed".to_owned(),
        )
        .unwrap();
    world.application.compare_and_commit_application(
        effects.finish().unwrap(),
        idempotency(key, intent),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
}
