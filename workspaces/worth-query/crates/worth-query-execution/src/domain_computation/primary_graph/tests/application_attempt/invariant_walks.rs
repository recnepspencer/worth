//! Installed complete walks preserve deep prepared authority and atomic denial.
mod api;
mod branch;
mod footprint;
mod locality;
mod observation;
mod operation_projection;
mod probe;
mod program;
mod resources;
mod rule;
pub(in crate::domain_computation::primary_graph) mod schema;
pub(in crate::domain_computation::primary_graph) mod world;

use crate::domain_computation::primary_graph::*;
use schema::{Node, WalkSchema};
use world::World;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

fn commit<Operation, Input>(
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
                        super::idempotency(key, key),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    ),
            );
        })
        .expect("the live installed request opens its ordinary advancement");
    outcome.expect("the actual compare-and-commit boundary returned")
}

fn assert_deep_callbacks(world: &World) {
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

fn exact_violation(outcome: WorthQueryApplicationCommitOutcome, expected: rule::Refusal) {
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("the deep domain violation must refuse the actual commit: {outcome:?}");
    };
    assert_eq!(
        denial
            .decode_invariant_refusal::<rule::RefusalBinding>()
            .unwrap(),
        expected,
        "an access, resource, setup or generic execution refusal is not this domain verdict"
    );
}
