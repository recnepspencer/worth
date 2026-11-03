//! Installed complete walks preserve deep prepared authority and atomic denial.

use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionPlacementForTest;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial;
use worth_query_execution::facade::domain_computation::advancement_requests_on_this_thread_for_test;
use worth_query_execution::facade::domain_computation::bound_advancement_requests_on_this_thread_for_test;
use worth_query_execution::domain_computation::primary_graph::denial;
use worth_query_execution::domain_computation::primary_graph::observations;
use worth_query_execution::facade::domain_computation::place_managed_computations_on_this_thread_for_test;
use worth_query_execution::domain_computation::primary_graph::root;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::schema::Node;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::schema::WalkSchema;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::world::World;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
pub fn commit<Operation, Input>(
    world: &World,
    request: &WorthQueryRequestScope,
    program: WorthQueryApplicationEffectProgram<WalkSchema, Operation, Input, Node>,
    key: u8,
) -> WorthQueryApplicationCommitOutcome
where
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
{
    let mut outcome = None;
    world
        .application
        .with_application_advancement(request, |phase| {
            outcome = Some(
                world
                    .application
                    .compare_and_commit_application_in_advancement(
                        &phase,
                        program,
                        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency(key, key),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    ),
            );
        })
        .expect("the live installed request opens its ordinary advancement");
    outcome.expect("the actual compare-and-commit boundary returned")
}
pub fn assert_deep_callbacks(world: &World) {
    let [prepared, evaluated, deep] = world.observation.contacts();
    assert!(
        prepared > 0,
        "the original candidate prepared a fresh scope"
    );
    assert!(evaluated > 0, "the actual installed rule evaluated");
    assert!(
        deep > 0,
        "the ordinary action reached the third-hop condition"
    );
}
pub fn exact_violation(
    outcome: WorthQueryApplicationCommitOutcome,
    expected: worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::rule::Refusal,
) {
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("the deep domain violation must refuse the actual commit: {outcome:?}");
    };
    assert_eq!(
        denial
            .decode_invariant_refusal::<worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::rule::RefusalBinding>()
            .unwrap(),
        expected,
        "an access, resource, setup or generic execution refusal is not this domain verdict"
    );
}

#[path = "declared_queries/mod.rs"]
pub mod declared_queries;
#[path = "program/mod.rs"]
pub mod program;
#[path = "reobservation/mod.rs"]
pub mod reobservation;
#[path = "world/mod.rs"]
pub mod world;
