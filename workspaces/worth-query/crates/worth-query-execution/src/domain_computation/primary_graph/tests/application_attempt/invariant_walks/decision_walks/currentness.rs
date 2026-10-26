//! Retain one real handler candidate across actual commits, including an unrelated twin.
use super::{
    super::{
        program,
        schema::*,
        world::{self, World},
    },
    binding::*,
};
use crate::domain_computation::primary_graph::application_entry::mutation::HandlerResult;
use crate::domain_computation::primary_graph::*;
use worth_execution::ExecutionAllocationPolicy as Allocation;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;
type Program = WorthQueryApplicationEffectProgram<WalkSchema, CheckWalk, CheckWalkInput, Node>;

fn prepare(
    world: &World,
    request: &WorthQueryRequestScope,
    key: &str,
) -> (Program, WorthQueryApplicationIdempotencyBinding) {
    world
        .application
        .with_application_advancement(request, |phase| {
            let host = phase.test_host_request();
            let execution = host
                .execution_request_for(world.application.product_runtime())
                .unwrap();
            let (admission, _) = program::authorization::authorize(
                world,
                &phase,
                &execution,
                request,
                0,
                CheckWalk::reference(),
                None,
            );
            let key_text = key.to_owned();
            let input = CheckWalkInput { root: 0 };
            let identities = ApplicationMutationIdentities::<WalkSchema, CheckWalkBinding>::encode(
                &key_text, &input,
            )
            .unwrap();
            let idempotency =
                WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities);
            // The fixture's actual authenticated binding maps Alice to principal identity1.
            let report = world
                .application
                .execute_mutation_handler_report::<CheckWalkBinding>(
                    &phase,
                    &identities,
                    &1_u64,
                    admission,
                    Allocation::SystemAllocation,
                );
            let WorthQueryMutationHandlerWork::Captured(work) = report.decision_work() else {
                panic!("the actual handler must execute")
            };
            assert!(work.handler_contacted());
            assert_eq!(work.projection_work().adjacency_lists_read(), 12);
            let HandlerResult::Completed(completed) = report.into_outcome().unwrap() else {
                panic!("handler prepares its actual candidate")
            };
            let (program, outcome) = completed.into_parts();
            assert_eq!(outcome, CheckWalkOutcome(4));
            (program, idempotency)
        })
        .unwrap()
}
fn publish(
    world: &World,
    request: &WorthQueryRequestScope,
    program: Program,
    key: WorthQueryApplicationIdempotencyBinding,
) -> WorthQueryApplicationCommitOutcome {
    let mut result = None;
    world
        .application
        .with_application_advancement(request, |phase| {
            result = Some(
                world
                    .application
                    .compare_and_commit_application_in_advancement(
                        &phase,
                        program,
                        key,
                        Allocation::SystemAllocation,
                    ),
            );
        })
        .unwrap();
    result.unwrap()
}
#[test]
fn retained_walk_handler_candidate_refuses_deep_leaf_and_nondiscovery_changes() {
    for edit in 0..3 {
        let graph = if edit == 2 {
            world::Graph {
                nodes: 6,
                active: false,
                blocked: vec![],
                edges: vec![(0, 1), (0, 2), (1, 3), (2, 3)],
            }
        } else {
            world::Graph::three_hops(false, vec![])
        };
        let world = world::install(graph);
        let request = super::super::super::super::fixture::live_scope();
        let (candidate, key) = prepare(&world, &request, "retained-walk");
        let change = match edit {
            0 => program::replace_next(&world, &request, 4),
            1 => program::link(&world, &request, 3, 4),
            _ => program::replace_next(&world, &request, 3),
        };
        assert!(matches!(
            super::super::commit(&world, &request, change, 70 + edit),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
        let outcome = publish(&world, &request, candidate, key);
        let WorthQueryApplicationCommitOutcome::Stale(stale) = outcome else {
            panic!("the complete dependency must be stale after edit{edit}: {outcome:?}")
        };
        assert_eq!(
            stale.stale_fact_count(),
            1,
            "three repeated logical walks retained one complete dependency"
        );
    }
}
#[test]
fn the_same_retained_walk_candidate_commits_after_an_unrelated_field_edit() {
    let world = world::install(world::Graph::three_hops(false, vec![]));
    let request = super::super::super::super::fixture::live_scope();
    let (candidate, key) = prepare(&world, &request, "unchanged-walk");
    let version = world
        .application
        .with_host_advancement(|phase| world.invariant.snapshot(&phase).unwrap().version())
        .unwrap();
    let unrelated = program::set_node_active(&world, &request, 5, true);
    assert!(matches!(
        super::super::commit(&world, &request, unrelated, 80),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let changed = world
        .application
        .with_host_advancement(|phase| world.invariant.snapshot(&phase).unwrap().version())
        .unwrap();
    assert_ne!(
        version, changed,
        "the ordinary unrelated write actually moved the graph head"
    );
    let outcome = publish(&world, &request, candidate, key);
    assert!(
        matches!(&outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "the same retained candidate stays current after the unrelated write: {outcome:?}"
    );
}
