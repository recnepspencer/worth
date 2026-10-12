use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

use super::{authenticated_principal, live_scope, resolved_account};
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, AccountStatus, ExactStatusRetentionOperation,
    RetainedStatusEffect, RetainedStatusNotice,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind;

#[test]
fn structured_effect_validation_precedes_retention_and_external_projection() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ExactStatusRetentionOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
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
                            .require_decision_field(projected, AccountStatus::reference())
                            .unwrap();
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("the fixture read owns its installed host call");
    let reads = world
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
        })
        .expect("the fixture read owns its installed host call");
    let mut effects = world
        .application
        .with_host_advancement(|phase| {
            reads
                .complete_projected_dependencies(
                    &phase,
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .begin_effect_program()
        })
        .expect("the fixture read owns its installed host call");

    let retained = effects
        .emit(
            RetainedStatusEffect::reference(),
            RetainedStatusNotice(String::new()),
        )
        .unwrap_err();
    assert_eq!(
        retained.kind(),
        WorthQueryApplicationAttemptDenialKind::InvalidEffectValue
    );
    let external = effects
        .emit_external(
            RetainedStatusEffect::reference(),
            RetainedStatusNotice(String::new()),
        )
        .unwrap_err();
    assert_eq!(
        external.kind(),
        WorthQueryApplicationAttemptDenialKind::InvalidEffectValue
    );
}
