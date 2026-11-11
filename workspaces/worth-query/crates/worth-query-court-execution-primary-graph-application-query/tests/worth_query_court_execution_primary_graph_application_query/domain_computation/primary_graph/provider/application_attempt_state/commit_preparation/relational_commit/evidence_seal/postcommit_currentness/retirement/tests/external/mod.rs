use crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::retirement::tests::*;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ActivityIdentity;
use worth_relational::facade::transactions::DeleteRelationIntent;
use worth_relational::facade::transactions::RelationMutationIntent;

#[test]
fn external_pair_and_adjacency_remain_current_after_owned_endpoint_retirement() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            let selected = world.selected_product(
                &(host_custody.phase_for_application_for_test(&world.application)).test_host_request_for_test(),
            );
            let owned = selected
                .resolve_entity(
                    &host_custody.phase_for_application_for_test(&world.application),
                    ActivityIdentity::at("ActivityIdentity"),
                    "activity-primary".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id_for_test();
            let (anchor, _, _) = reads(
                &host_custody.phase_for_application_for_test(&world.application),
                execution,
                &world,
                "account-1",
            );
            let graph = (*world.application.runtime_for_test()).primary_graph_for_test().unwrap();
            let relation = *(*graph
                .layout_for_test())
                .relation(AccountAllActivity::reference().name())
                .unwrap()
                .kind_for_test();
            let kind = (*graph
                .layout_for_test())
                .entity_kind_for_test(ActivityIdentity::at("ActivityIdentity").entity())
                .unwrap();
            let relations = graph.integration_handle_for_test().with_runtime(&phase.borrowed_request_for_test(), |runtime| {
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::adjacency::observe_adjacency(
                    runtime,
                    execution,
                    selected.application_basis_for_test().snapshot_handle(),
                    relation,
                    anchor,
                    WorthQueryApplicationAdjacencyDirection::Outgoing,
                    16,
                ).unwrap()
                .unwrap()
            });
            let removed = *relations
                .iter()
                .find(|entry| (*entry.to_for_test()) == owned)
                .unwrap()
                .relation_id_for_test();
            let retained = *relations
                .iter()
                .find(|entry| (*entry.to_for_test()) != owned)
                .unwrap()
                .relation_id_for_test();
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
                        .filter(|entry| (*entry.to_for_test()) == owned)
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
                &host_custody.borrowed_request_for_test(),
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
            let retired = rebase_current(
                &host_custody.phase_for_application_for_test(&world.application),
                &host_custody.borrowed_request_for_test(),
                &world,
                facts,
                &BTreeSet::from([owned]),
            )
            .unwrap();
            assert!(retired[..3]
                .iter()
                .all(|fact| matches!(fact, Fact::RetiredOutputEntity { .. })));
            assert!(matches!(retired[3], Fact::Relation { from, .. } if from == anchor));
            assert!(
                matches!(retired[4], Fact::Adjacency { anchor: actual, .. } if actual == anchor)
            );
            let native = normal_rebase(
                &host_custody.phase_for_application_for_test(&world.application),
                execution,
                &world,
                retired,
            );
            assert!(
                selection(
                    &host_custody.phase_for_application_for_test(&world.application),
                    execution,
                    &world,
                    &native
                ),
                "the external anchor compares its real postcommit adjacency"
            );
            publish_relational_mutation(
                &host_custody.borrowed_request_for_test(),
                &world,
                WorkerIntentBatch::new("change-external-adjacency").push(MutationIntent::Relation(
                    RelationMutationIntent::Delete(DeleteRelationIntent {
                        relation_id: retained,
                    }),
                )),
            );
            assert!(
                !selection(
                    &host_custody.phase_for_application_for_test(&world.application),
                    execution,
                    &world,
                    &native
                ),
                "a later change to the external anchor's adjacency invalidates the output"
            );
        })
        .expect("the fixture block owns its installed host call")
}

#[test]
fn a_retired_entity_changes_each_revision_fact_read_without_its_entity_fact() {
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::movement::Movement;
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            let (entity, _, facts) = reads(
                &host_custody.phase_for_application_for_test(&world.application),
                execution,
                &world,
                "account-1",
            );
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
            let probe = |fact: &Fact| {
                let selected = world.selected_product(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
                );
                let graph = (*world.application.runtime_for_test())
                    .primary_graph_for_test()
                    .unwrap();
                graph.integration_handle_for_test().with_runtime(
                    &phase.borrowed_request_for_test(),
                    |runtime| {
                        fact.source_currentness_in_for_test(
                            runtime,
                            execution,
                            selected.application_basis_for_test().snapshot_handle(),
                        )
                    },
                )
            };
            for fact in &revisions {
                assert_eq!(
                    probe(fact).unwrap().unwrap().movement_for_test(),
                    Movement::Unmoved,
                    "{fact:?}"
                );
            }
            delete(&host_custody.borrowed_request_for_test(), &world, entity);
            // Source facts are kept in canonical key order, so a revision fact can be
            // read before the lifecycle fact of the same entity. Each answers alone.
            for fact in &revisions {
                let movement = probe(fact).unwrap().unwrap();
                assert_eq!(
                    movement.movement_for_test(),
                    Movement::Moved,
                    "a retired entity changed this fact: {fact:?}"
                );
            }
        })
        .expect("the fixture block owns its installed host call")
}
