//! A released typed successor refuses at the producer's resume boundary.

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::performed::PerformedMembers;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::required_continuations::TypedRequiredSuccessor;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::retained_output_capacity::RetainedFamily;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use std::cell::RefCell;
use std::time::Duration;

#[test]
fn absent_admitted_demand_refuses_with_required_successor_state_unavailable() {
    let world = installed_authorization_world(true);
    let scope = live_scope();
    let principal = world.authenticate("alice", Duration::from_secs(60), &scope);
    // This is the emptied state after custody releases its admitted demand.
    // No fabricated interest, provenance or authority participates in the call.
    let mut successor = TypedRequiredSuccessor::<IdentityExecutionSchema, RetainedFamily>::new_for_test(None, None, None, None);
    world
        .application
        .with_host_advancement(|phase| {
            let runtime = &world.application;
            let execution = phase.execution_request_for_for_test(runtime.product_runtime()).unwrap();
            let denial = successor
                .run_again_for_test(
                    &execution,
                    &phase,
                    runtime,
                    &principal,
                    &scope,
                    runtime.current_world(),
                    &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::invocation::ProducerInvocationAuthority::from(
                        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerCommitAuthority::Ordinary,
                    ),
                    &RefCell::new(PerformedMembers::start()),
                )
                .err()
                .expect("an absent successor demand must refuse");
            assert_eq!(
                denial.kind(),
                worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind::RequiredSuccessorStateUnavailable
            );
        })
        .expect("the installed host admits the resume call");
}
