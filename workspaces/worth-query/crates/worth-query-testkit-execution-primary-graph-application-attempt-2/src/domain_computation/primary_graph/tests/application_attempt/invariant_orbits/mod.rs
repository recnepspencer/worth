//! Installed orbit APIs on static schema profiles, through ordinary commits.

use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
use worth_query_execution::domain_computation::primary_graph::denial;
use worth_query_execution::domain_computation::primary_graph::root;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::Node;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::OrbitSchema;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::World;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
pub fn commit<const O: bool>(
    world: &World<O>,
    request: &WorthQueryRequestScope,
    program: WorthQueryApplicationEffectProgram<
        OrbitSchema<O>,
        crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::ReplaceNext,
        crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::ReplaceNextInput,
        Node,
    >,
    key: u8,
) -> WorthQueryApplicationCommitOutcome {
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
        .unwrap();
    outcome.unwrap()
}
pub fn assert_deep_callbacks(world: &World) {
    let [prepared, evaluated, deep] = world.observation.contacts();
    assert!(
        prepared > 0 && evaluated > 0 && deep > 0,
        "real scope, evaluation and deep member required"
    );
}
pub fn exact_violation(
    outcome: WorthQueryApplicationCommitOutcome,
    expected: crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::Refusal,
) {
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("actual domain violation required: {outcome:?}");
    };
    assert_eq!(
        denial
            .decode_invariant_refusal::<crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::RefusalBinding>()
            .unwrap(),
        expected
    );
}

#[path = "component_invariants/mod.rs"]
pub mod component_invariants;
#[path = "decision/mod.rs"]
pub mod decision;
#[path = "declared_queries/mod.rs"]
pub mod declared_queries;
#[path = "probe/mod.rs"]
pub mod probe;
#[path = "program/mod.rs"]
pub mod program;
#[path = "schema/mod.rs"]
pub mod schema;
#[path = "world/mod.rs"]
pub mod world;
