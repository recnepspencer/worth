use super::*;
use crate::domain_computation::primary_graph::tests::fixture::ActivityIdentity;
use worth_relational::facade::transactions::{DeleteRelationIntent, RelationMutationIntent};

#[test]
fn external_pair_and_adjacency_remain_current_after_owned_endpoint_retirement() {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let owned = selected
        .resolve_entity(
            ActivityIdentity::reference(),
            "activity-primary".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    let (anchor, _, _) = reads(&world, "account-1");
    let graph = world.application.runtime.primary_graph().unwrap();
    let relation = graph
        .layout
        .relation(AccountAllActivity::reference().name())
        .unwrap()
        .kind;
    let kind = graph
        .layout
        .entity_kind(ActivityIdentity::reference().entity())
        .unwrap();
    let relations = graph.integration_handle().with_runtime(|runtime| {
        crate::domain_computation::primary_graph::application_attempt::observe_adjacency(
            runtime,
            selected.application_basis().snapshot_handle(),
            relation,
            anchor,
            WorthQueryApplicationAdjacencyDirection::Outgoing,
            16,
        )
        .unwrap()
    });
    let removed = relations
        .iter()
        .find(|entry| entry.to == owned)
        .unwrap()
        .relation_id;
    let retained = relations
        .iter()
        .find(|entry| entry.to != owned)
        .unwrap()
        .relation_id;
    let facts = vec![
        Fact::Entity {
            entity_id: owned,
            kind,
        },
        Fact::SourceEntity { entity_id: owned },
        Fact::Adjacency {
            anchor: owned,
            relation_kind: relation,
            direction: WorthQueryApplicationAdjacencyDirection::Incoming,
            maximum_work_units: 16,
            relations: relations
                .iter()
                .filter(|entry| entry.to == owned)
                .cloned()
                .collect(),
        },
        Fact::Relation {
            from: anchor,
            to: owned,
            relation_kind: relation,
            matching_relations: vec![removed],
        },
        Fact::Adjacency {
            anchor,
            relation_kind: relation,
            direction: WorthQueryApplicationAdjacencyDirection::Outgoing,
            maximum_work_units: 16,
            relations,
        },
    ];
    drop(selected);
    publish_relational_mutation(
        &world,
        WorkerIntentBatch::new("retire-owned-related-output")
            .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                DeleteRelationIntent {
                    relation_id: removed,
                },
            )))
            .push(MutationIntent::Entity(EntityMutationIntent::Delete(
                DeleteEntityIntent { entity_id: owned },
            ))),
    );
    let retired = rebase_current(&world, facts, &BTreeSet::from([owned])).unwrap();
    assert!(retired[..3]
        .iter()
        .all(|fact| matches!(fact, Fact::RetiredOutputEntity { .. })));
    assert!(matches!(retired[3], Fact::Relation { from, .. } if from == anchor));
    assert!(matches!(retired[4], Fact::Adjacency { anchor: actual, .. } if actual == anchor));
    let native = normal_rebase(&world, retired);
    assert!(
        selection(&world, &native),
        "the external anchor compares its real postcommit adjacency"
    );
    publish_relational_mutation(
        &world,
        WorkerIntentBatch::new("change-external-adjacency").push(MutationIntent::Relation(
            RelationMutationIntent::Delete(DeleteRelationIntent {
                relation_id: retained,
            }),
        )),
    );
    assert!(
        !selection(&world, &native),
        "a later change to the external anchor's adjacency invalidates the output"
    );
}

#[test]
fn a_retired_entity_changes_each_revision_fact_read_without_its_entity_fact() {
    use crate::domain_computation::primary_graph::application_attempt::{
        Movement, WorthQuerySourceCurrentnessFailure,
    };
    let world = installed_authorization_world(true);
    let (entity, _, facts) = reads(&world, "account-1");
    let revisions = facts
        .into_iter()
        .filter(|fact| {
            matches!(
                fact,
                Fact::SourceAspectRevision { .. }
                    | Fact::SourceFieldRevision { .. }
                    | Fact::SourceAdjacencyRevision { .. }
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(revisions.len(), 3);
    let probe = |fact: &Fact, maximum_work| {
        let selected = world.selected_product();
        let graph = world.application.runtime.primary_graph().unwrap();
        graph.integration_handle().with_runtime(|runtime| {
            fact.source_currentness_in(
                runtime,
                selected.application_basis().snapshot_handle(),
                maximum_work,
            )
        })
    };
    for fact in &revisions {
        assert_eq!(
            probe(fact, 64).unwrap().0.movement(),
            Movement::Unmoved,
            "{fact:?}"
        );
    }
    delete(&world, entity);
    // Source facts are kept in canonical key order, so a revision fact can be
    // read before the lifecycle fact of the same entity. Each answers alone.
    for fact in &revisions {
        let (movement, work) = probe(fact, 64).unwrap();
        assert_eq!(
            movement.movement(),
            Movement::Moved,
            "a retired entity changed this fact: {fact:?}"
        );
        assert_eq!(
            probe(fact, work - 1),
            Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded),
            "the liveness probe is admitted work: {fact:?}"
        );
    }
}
