//! A transferred source selects its own publications, leaving B's other output current.
use super::publication_fixture::{self, PublishedModel};
use super::transfer_fixture::{basis, move_to};
use super::*;
use worth_relational::facade::{branch::AdmittedRelationalBranchBasis, mvcc::*};
#[test]
fn source_transfer_restores_its_two_output_families_and_owned_edge_without_selecting_bs_other_publication(
) {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let world = PublishedModel::new(execution);
    let mut related = world.meaning.clone();
    related.family = "related-output".into();
    let (related_output, related_edge) = publication_fixture::publish_related(
        execution,
        &world.runtime,
        &world.owner,
        &world.installed,
        &related,
        "related-output",
    );
    let at_a = basis(&world.runtime);
    let descriptive = world
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
    let ownership_edge = descriptive
        .current_relations()
        .values()
        .find(|edge| {
            edge.kind_id == KindId(2) && edge.source == world.model && edge.target == world.source
        })
        .unwrap()
        .relation_id;
    drop(descriptive);
    let b = create_entity(execution, &world.runtime, "b");
    let other_source = member(execution, &world.runtime, "b-other-source");
    create_relation(
        execution,
        &world.runtime,
        b,
        other_source,
        "b-other-source-owner",
    );
    let mut other_meaning = world.meaning.clone();
    other_meaning.source_scope = other_source;
    let other_output = publication_fixture::publish(
        execution,
        &world.runtime,
        &world.owner,
        &world.installed,
        &other_meaning,
        "b-other-output",
    );
    let now = basis(&world.runtime);
    let other_coordinate = world
        .runtime
        .prepare_historical_output_selection(
            execution,
            &world.installed,
            &now,
            &now,
            &other_meaning,
        )
        .unwrap()
        .publication()
        .authority()
        .publication_coordinate();
    move_to(execution, &world.runtime, ownership_edge, b, world.source);
    let at_b = basis(&world.runtime);
    for (selected, owner) in [(&at_a, world.model), (&at_b, b), (&at_a, world.model)] {
        let committed = select_outputs(execution, &world, selected);
        let after = basis(&world.runtime);
        let publications = committed
            .required_output_publications(&world.installed, execution)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(publications.len(), 2);
        assert!(publications.iter().all(|p| p.is_historical_selection()
            && p.meaning().source_scope == world.source
            && p.authority().publication_coordinate().0 == committed.commit.commit_id.0));
        for id in [world.source, world.output, related_output] {
            assert_eq!(
                world
                    .runtime
                    .observe_model_owner(execution, &after, &ownership(), id)
                    .unwrap(),
                Some(owner)
            );
        }
        let mut read = begin(&world.runtime);
        let edge = read.read_relation(execution, related_edge).unwrap();
        let edge = edge.base().unwrap();
        assert_eq!((edge.source, edge.target), (world.source, related_output));
        assert_eq!(
            edge.lifecycle,
            worth_relational::facade::storage::RecordLifecycleState::Live
        );
        let untouched = world
            .runtime
            .prepare_historical_output_selection(
                execution,
                &world.installed,
                &after,
                &after,
                &other_meaning,
            )
            .unwrap();
        assert_eq!(
            untouched.publication().authority().publication_coordinate(),
            other_coordinate
        );
        assert!(!untouched.publication().is_historical_selection());
        assert_eq!(
            untouched
                .publication()
                .members()
                .map(|(_, _, _, id)| id)
                .collect::<Vec<_>>(),
            [other_output]
        );
        assert_eq!(
            world
                .runtime
                .observe_model_owner(execution, &after, &ownership(), other_output)
                .unwrap(),
            Some(b)
        );
        assert!(!committed.changed_records.contains(&RecordRef::Entity(b)));
        assert!(!committed
            .changed_records
            .contains(&RecordRef::Entity(other_source)));
        assert!(!committed
            .changed_records
            .contains(&RecordRef::Entity(other_output)));
        release_test_commit_snapshot(&world.runtime, &committed);
    }
}
pub(super) fn select_outputs(
    execution: &ExecutionRequest<'_, '_>,
    world: &PublishedModel,
    selected: &AdmittedRelationalBranchBasis,
) -> CommitResult {
    let graph = world
        .runtime
        .prepare_owned_model_graph_selection(
            execution,
            &basis(&world.runtime),
            selected,
            &ownership(),
            &ownership(),
            world.model,
        )
        .unwrap();
    let selections = world
        .runtime
        .prepare_historical_output_selections(execution, &world.installed, &graph)
        .unwrap();
    assert_eq!(selections.len(), 2);
    let admission = world
        .owner
        .admit_historical_output_selection(selections.iter().map(|s| s.publication().authority()))
        .unwrap();
    let port = world.runtime.preparation_port();
    let input = port
        .admit_owned_graph_selection(execution, begin(&world.runtime), graph)
        .unwrap();
    let candidate = port
        .prepare_owned_graph_selection(
            execution,
            input,
            Some((
                &world.installed,
                RequiredOutputPublication::HistoricalSelection {
                    admission,
                    selections,
                },
            )),
        )
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = world
        .runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("transferred publications must select atomically");
    };
    world
        .runtime
        .settle_performed_publication(performed)
        .unwrap()
}

#[test]
fn selected_transferred_publication_restores_retired_original_members_and_retires_the_replacement()
{
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
    let initial = basis(&world.runtime);
    let graph = world
        .runtime
        .prepare_owned_model_graph_selection(
            execution,
            &initial,
            &initial,
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
    let old_at_b = basis(&world.runtime);
    move_to(execution, &world.runtime, edge, world.model, world.source);
    let (replacement, replacement_edge) =
        publication_fixture::replace_related(execution, &world, &related, old, old_edge);
    let new_at_a = basis(&world.runtime);
    assert_ne!(replacement, old);
    for (selected, live, retired, live_edge, retired_edge, owner) in [
        (&old_at_b, old, replacement, old_edge, replacement_edge, b),
        (
            &new_at_a,
            replacement,
            old,
            replacement_edge,
            old_edge,
            world.model,
        ),
        (&old_at_b, old, replacement, old_edge, replacement_edge, b),
    ] {
        let committed = select_outputs(execution, &world, selected);
        let mut read = begin(&world.runtime);
        assert_eq!(
            read.read_entity(execution, live)
                .unwrap()
                .base()
                .unwrap()
                .entity_id,
            live
        );
        assert!(read
            .read_entity(execution, retired)
            .unwrap()
            .base()
            .is_none());
        assert_eq!(
            read.read_relation(execution, live_edge)
                .unwrap()
                .base()
                .unwrap()
                .target,
            live
        );
        assert_eq!(
            read.read_relation(execution, retired_edge)
                .unwrap()
                .base()
                .unwrap()
                .lifecycle,
            worth_relational::facade::storage::RecordLifecycleState::DeletedRetained
        );
        let current = basis(&world.runtime);
        assert_eq!(
            world
                .runtime
                .observe_model_owner(execution, &current, &ownership(), live)
                .unwrap(),
            Some(owner)
        );
        release_test_commit_snapshot(&world.runtime, &committed);
    }
}
