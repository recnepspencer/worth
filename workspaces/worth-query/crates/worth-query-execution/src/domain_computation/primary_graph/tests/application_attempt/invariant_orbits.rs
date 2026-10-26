//! Installed orbit APIs on static schema profiles, through ordinary commits.
mod api;
pub(in crate::domain_computation::primary_graph) mod decision;
pub(in crate::domain_computation::primary_graph) mod declared_queries;
mod lineage;
mod probe;
pub(in crate::domain_computation::primary_graph) mod program;
mod projection;
pub(in crate::domain_computation::primary_graph) mod schema;
pub(in crate::domain_computation::primary_graph) mod world;
use crate::domain_computation::primary_graph::*;
use schema::{Node, OrbitSchema};
use world::World;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

fn commit<const O: bool>(
    world: &World<O>,
    request: &WorthQueryRequestScope,
    program: WorthQueryApplicationEffectProgram<
        OrbitSchema<O>,
        schema::ReplaceNext,
        schema::ReplaceNextInput,
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
                        super::idempotency(key, key),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    ),
            );
        })
        .unwrap();
    outcome.unwrap()
}
fn assert_deep_callbacks(world: &World) {
    let [prepared, evaluated, deep] = world.observation.contacts();
    assert!(
        prepared > 0 && evaluated > 0 && deep > 0,
        "real scope, evaluation and deep member required"
    );
}
fn exact_violation(outcome: WorthQueryApplicationCommitOutcome, expected: probe::Refusal) {
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("actual domain violation required: {outcome:?}");
    };
    assert_eq!(
        denial
            .decode_invariant_refusal::<probe::RefusalBinding>()
            .unwrap(),
        expected
    );
}
