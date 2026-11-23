//! The decision reader's `name_of` through a real installed operation
//! projection and `complete_projected_dependencies`. Installed Query
//! runtimes bind no persistent-name scheme into Relational yet, so their
//! records carry no names: the read refuses `RecordNotNamed`, keys the
//! record's entity fact, and the projection still completes. An entity kind
//! the operation does not declare as a decision read cannot be named at all:
//! `name_of` and `resolve` require `Entity: OperationReads<Operation>`.

use std::time::Duration;

use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;
use worth_query_execution::facade::application_naming::{
    ExecutionNameReadDenial, WorthQueryDecisionNameDenial,
};
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::{
    schema_types::live_scope, world_installation::installed_authorization_world,
    TouchAccountOperation,
};

#[test]
fn decision_name_of_on_an_unnamed_record_keys_the_record_and_completes() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            let custody = phase.test_host_request_for_test();
            let execution = custody.borrowed_request_for_test();
            let branch = || {
                world
                    .application
                    .select_product_branch_for_test(
                        &execution,
                        world.application.product_runtime().default_branch_for_test(),
                    )
                    .expect("the product branch remains admitted")
            };
            let external = world.authenticate("alice", Duration::from_secs(60), &request);
            let actor = branch()
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let account = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account(
                &custody, &world, "open", &request,
            );
            let operation = world
                .application
                .installed_schema()
                .installed_operation(TouchAccountOperation::reference())
                .unwrap();
            world
                .selected_product(&custody)
                .authorize_operation(&phase, &actor, &account, &operation, Default::default(), &request)
                .unwrap()
        })
        .expect("the fixture owns its installed host call");
    let (_, projection, _) = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, account| {
                        assert!(matches!(
                            reader.name_of(account),
                            Err(WorthQueryDecisionNameDenial::Read(
                                ExecutionNameReadDenial::RecordNotNamed { .. }
                            ))
                        ));
                    },
                    AllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("the fixture owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    AllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .complete_projected_dependencies(&phase, AllocationPolicy::SystemAllocation)
                .expect("a stable name refusal keyed on the record seals and completes")
        })
        .expect("the fixture owns its installed host call");
}
