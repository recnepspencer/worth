//! Producer successors after selection remain within the next inverse's custody.
use super::publication_fixture::{self, PublishedModel};
use super::transfer_fixture::{basis, move_to};
use super::transfer_publications::select_outputs;
use super::*;
#[test]
fn a_producer_replacement_after_transfer_is_retired_by_the_next_inverse_selection() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let world = PublishedModel::new(execution);
    let mut related = world.meaning.clone();
    related.family = "related-output".into();
    let (old, old_edge) = publication_fixture::publish_related(
        execution,
        &world.runtime,
        &world.owner,
        &world.installed,
        &related,
        "older-result",
    );
    let b = create_entity(execution, &world.runtime, "b");
    let at_a = basis(&world.runtime);
    let graph = world
        .runtime
        .prepare_owned_model_graph_selection(
            execution,
            &at_a,
            &at_a,
            &ownership(),
            &ownership(),
            world.model,
        )
        .unwrap();
    let edge = graph
        .current_relations()
        .values()
        .find(|edge| {
            edge.kind_id == KindId(2) && edge.source == world.model && edge.target == world.source
        })
        .unwrap()
        .relation_id;
    drop(graph);
    move_to(execution, &world.runtime, edge, b, world.source);
    let at_b = basis(&world.runtime);
    for selected in [&at_a, &at_b] {
        let committed = select_outputs(execution, &world, selected);
        release_test_commit_snapshot(&world.runtime, &committed);
    }
    let (replacement, replacement_edge) =
        publication_fixture::replace_related(execution, &world, &related, old, old_edge);
    let current = basis(&world.runtime);
    assert_eq!(
        world
            .runtime
            .observe_model_owner(execution, &current, &ownership(), replacement)
            .unwrap(),
        Some(b)
    );
    let restored = select_outputs(execution, &world, &at_a);
    let mut read = begin(&world.runtime);
    assert_eq!(
        read.read_entity(execution, old)
            .unwrap()
            .base()
            .unwrap()
            .entity_id,
        old
    );
    assert_eq!(
        read.read_relation(execution, old_edge)
            .unwrap()
            .base()
            .unwrap()
            .target,
        old
    );
    assert!(read
        .read_entity(execution, replacement)
        .unwrap()
        .base()
        .is_none());
    assert_eq!(
        read.read_relation(execution, replacement_edge)
            .unwrap()
            .base()
            .unwrap()
            .lifecycle,
        worth_relational::facade::storage::RecordLifecycleState::DeletedRetained
    );
    let after = basis(&world.runtime);
    assert_eq!(
        world
            .runtime
            .observe_model_owner(execution, &after, &ownership(), old)
            .unwrap(),
        Some(world.model)
    );
    release_test_commit_snapshot(&world.runtime, &restored);
}
