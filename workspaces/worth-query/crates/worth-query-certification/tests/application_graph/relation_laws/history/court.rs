//! Authored actions, receipt coordinates and independent committed identities form the oracle.
use super::super::{
    creation::{Create, CreateNodeInput},
    feeding::FeedsChange,
    installation::*,
    operation::*,
    outcome::committed,
    read,
};
use super::{access::*, *};
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use std::marker::PhantomData;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationRequestExt,
    },
    application_invariants::{EntityId, RelationId},
    primary_graph::*,
    runtime::ExecutionAllocationPolicy,
};

#[derive(Debug, Eq, PartialEq)]
struct Event {
    commit: u64,
    entity: Option<EntityId>,
    relation: Option<RelationId>,
    structure: RecordStructuralChange,
}
fn events(page: &WorthQueryApplicationHistoryPage) -> Vec<Event> {
    assert!(page.source_visits() <= 1);
    assert!(page.entries().count() <= 1);
    assert!(page.newly_owned_bytes() <= limits().maximum_owned_bytes.get());
    page.entries()
        .map(|entry| {
            assert!(entry.changed_aspects().count() <= 1);
            if entry.structural_change() == RecordStructuralChange::Created {
                assert_eq!(entry.changed_aspects().count(), 1);
            }
            Event {
                commit: entry.commit_id().0,
                entity: entry.entity_id(),
                relation: entry.relation_id(),
                structure: entry.structural_change(),
            }
        })
        .collect()
}
fn receipt(outcome: Outcome<GraphDenial, Edited>) -> WorthQueryApplicationCommitReceipt {
    match outcome {
        Outcome::Committed { receipt, .. } => receipt,
        other => panic!("authored transaction must commit: {other:?}"),
    }
}

fn physical(world: &World<Payload>) -> Vec<(u64, RelationId, u64)> {
    let completed = world
        .application
        .runtime()
        .with_host_advancement(|phase| {
            world
                .projection
                .project(
                    &phase,
                    |reader| {
                        let source = reader.resolve_entity(NodeKey::reference(), 0).unwrap();
                        let mut ids = reader
                            .relations_from(Precedes::reference(), &source)
                            .unwrap()
                            .into_iter()
                            .map(|edge| {
                                (
                                    reader.field(edge.to(), NodeKey::reference()).unwrap(),
                                    edge.relation_id(),
                                    reader
                                        .relation_field(&edge, EdgeWeight::reference())
                                        .unwrap(),
                                )
                            })
                            .collect::<Vec<_>>();
                        ids.sort();
                        ids
                    },
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    let (ids, snapshot, _) = completed.into_parts();
    drop(snapshot);
    ids
}

#[test]
fn disclosed_physical_occurrences_and_deleted_generations_page_without_losing_root_import_or_position(
) {
    let mut world = install::<Payload>(4, &[]);
    let runtime = world.application.runtime();
    let root = observe(&world);
    let bootstrap = runtime
        .with_host_advancement(|phase| {
            root.select_on(&phase, runtime)
                .unwrap()
                .product()
                .relational_basis_descriptor()
                .clone()
        })
        .unwrap();
    let root_commit = bootstrap
        .reference()
        .target()
        .as_basis()
        .unwrap()
        .selected_commit_id();
    let identities = read::committed(&world).identities;
    let created = receipt(world.edit(10, &[(0, 1), (0, 2), (0, 3)]));
    let records = physical(&world);
    assert!(records.iter().all(|(_, _, weight)| *weight == 10));
    let mut ids = records.iter().map(|(_, id, _)| *id).collect::<Vec<_>>();
    ids.sort();
    let retired_ids = records
        .iter()
        .filter(|(key, _, _)| *key == 1)
        .map(|(_, id, _)| *id)
        .collect::<Vec<_>>();
    let selected_id = records.iter().find(|(key, _, _)| *key == 2).unwrap().1;
    assert_eq!(
        ids.len(),
        3,
        "the ordinary installed relation admits distinct physical occurrences"
    );
    assert_ne!(ids[0], ids[1]);
    let before_deletion = observe(&world);
    let entities = capture::<Entities>(&world, 0);
    let relations = capture::<Relations>(&world, 0);
    let wrong_filter = capture::<Relations>(&world, 2);
    let filtered = super::filtered::capture(&world, 0);
    let deleted = receipt(world.edit_feeds(11, 1, FeedsChange::DeleteSource));
    assert_eq!(read::identity(&world, 1), None);
    let request = request_scope();
    let external = authenticate_operator(world.application.installed_schema(), &request);
    let replacement = receipt(
        world
            .application
            .runtime()
            .request(&external, &request)
            .on_branch(world.application.current_world())
            .advanced()
            .mutate(Create::<Payload> {
                input: CreateNodeInput {
                    key: 1,
                    context: 0,
                    in_context: true,
                    link: false,
                    backlink: false,
                    self_cycle: false,
                },
                marker: PhantomData,
            })
            .without_source()
            .idempotency(&12_u64)
            .execute_in_program(
                &world.application,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap(),
    );
    let new_entity = read::identity(&world, 1).unwrap();
    assert_ne!(
        new_entity, identities[&1],
        "the same authored key is a new exact generation"
    );
    let replacement_edge = receipt(world.edit(13, &[(0, 1)]));
    let fresh = physical(&world)
        .into_iter()
        .find(|(key, _, _)| *key == 1)
        .unwrap();
    assert_eq!(fresh.2, 13);
    assert!(
        !ids.contains(&fresh.1),
        "replacement has its own physical relation occurrence"
    );
    let selected = observe(&world);
    let mut cursor = open(&world, &relations, &selected);
    let foreign = install::<Payload>(4, &[]);
    let foreign_commit = receipt(foreign.edit(10, &[(0, 1), (0, 2), (0, 3)]));
    let foreign_subjects = capture::<Relations>(&foreign, 0);
    let foreign_root = observe(&foreign);
    let mut foreign_cursor = open(&foreign, &foreign_subjects, &foreign_root);
    assert!(matches!(
        page(
            &world,
            &relations,
            &mut foreign_cursor,
            &selected,
            &request_scope()
        )
        .unwrap(),
        Err(WorthQueryApplicationHistoryDenial::InvalidContinuation)
    ));
    drop((
        foreign_cursor,
        foreign_subjects,
        foreign_root,
        foreign_commit,
    ));
    foreign.close();
    let mut actual = Vec::new();
    let mut checked_stops = false;
    let mut empty_incomplete = false;
    loop {
        let observed_page = page(&world, &relations, &mut cursor, &selected, &request_scope())
            .unwrap()
            .unwrap();
        empty_incomplete |=
            observed_page.entries().next().is_none() && !observed_page.is_complete();
        actual.extend(events(&observed_page));
        if !checked_stops && actual.len() == 1 {
            assert!(matches!(
                page(
                    &world,
                    &wrong_filter,
                    &mut cursor,
                    &selected,
                    &request_scope()
                )
                .unwrap(),
                Err(WorthQueryApplicationHistoryDenial::InvalidContinuation)
            ));
            assert!(matches!(
                page(
                    &world,
                    &relations,
                    &mut cursor,
                    &before_deletion,
                    &request_scope()
                )
                .unwrap(),
                Err(WorthQueryApplicationHistoryDenial::InvalidContinuation)
            ));
            let cancellation = WorthQueryCancellationSource::new();
            let stopped = WorthQueryRequestScope::new(None, cancellation.token());
            cancellation.cancel();
            assert!(matches!(page(&world, &relations, &mut cursor, &selected, &stopped), Err(worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled))));
            checked_stops = true;
        }
        if observed_page.is_complete() {
            break;
        }
    }
    assert!(empty_incomplete && checked_stops);
    let expected = [
        (
            deleted.commit_reference().commit_id.0,
            RecordStructuralChange::RetainedForAudit,
        ),
        (
            created.commit_reference().commit_id.0,
            RecordStructuralChange::Created,
        ),
    ]
    .into_iter()
    .flat_map(|(commit, structure)| {
        let selected = if structure == RecordStructuralChange::RetainedForAudit {
            &retired_ids
        } else {
            &ids
        };
        selected.iter().map(move |id| Event {
            commit,
            entity: None,
            relation: Some(*id),
            structure,
        })
    })
    .collect::<Vec<_>>();
    assert_eq!(actual, expected, "reversed authored publications and actual physical IDs, with real distinct physical records in one commit");
    drop(cursor);
    let mut cursor = open(&world, &filtered, &selected);
    let mut filtered_events = Vec::new();
    loop {
        let result = page(&world, &filtered, &mut cursor, &selected, &request_scope())
            .unwrap()
            .unwrap();
        filtered_events.extend(events(&result));
        if result.is_complete() {
            break;
        }
    }
    assert_eq!(
        filtered_events,
        vec![Event {
            commit: created.commit_reference().commit_id.0,
            entity: None,
            relation: Some(selected_id),
            structure: RecordStructuralChange::Created
        }]
    );
    drop((cursor, filtered));
    let mut cursor = open(&world, &entities, &selected);
    let mut actual = Vec::new();
    let mut imported = false;
    loop {
        let observed_page = page(&world, &entities, &mut cursor, &selected, &request_scope())
            .unwrap()
            .unwrap();
        imported |= observed_page.is_root_import() && observed_page.entries().next().is_some();
        actual.extend(events(&observed_page));
        if observed_page.is_complete() {
            break;
        }
    }
    let mut original_entities = vec![
        identities[&0],
        identities[&1],
        identities[&2],
        identities[&3],
    ];
    original_entities.sort();
    let mut expected = vec![Event {
        commit: deleted.commit_reference().commit_id.0,
        entity: Some(identities[&1]),
        relation: None,
        structure: RecordStructuralChange::Deleted,
    }];
    expected.extend(original_entities.into_iter().map(|id| Event {
        commit: root_commit,
        entity: Some(id),
        relation: None,
        structure: RecordStructuralChange::Created,
    }));
    assert_eq!(
        actual, expected,
        "replacement must not lend record identity to the captured generation"
    );
    assert!(
        imported,
        "a real World root imports exactly its actual canonical Native component"
    );
    drop(cursor);
    super::custody::probe(&world, &selected);
    drop((
        entities,
        relations,
        wrong_filter,
        root,
        before_deletion,
        selected,
        created,
        deleted,
        replacement,
        replacement_edge,
    ));
    committed(world.edit(14, &[(1, 2)]));
    committed(world.edit(15, &[(2, 3)]));
    assert!(
        world
            .application
            .runtime()
            .world_history_snapshot_for_test()
            .installed_commits()
            <= 2,
        "final owners no longer hold old product history"
    );
    world.close();
}
