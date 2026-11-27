//! The adjacency comparison stays its own arm beside a relaxed retirement arm.
use super::retirement::payload;
use worth_query_execution::facade::domain_computation::WorthQueryCorrectionDerivationDenial as Denial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationAdjacencyDirection as DecisionDirection;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact as Fact;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::adjacency::observe_adjacency;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::payload::FactPayload;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::product_operation::correction_source::restored_support::eligibility::compare;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::adjacency::rebase_decision_adjacency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountOwner;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ActivityIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::PrincipalIdentityField;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_relational::facade::transactions::DeleteEntityIntent;
use worth_relational::facade::transactions::DeleteRelationIntent;
use worth_relational::facade::transactions::EntityMutationIntent;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::RelationMutationIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;

#[test]
fn paired_support_keeps_the_adjacency_comparison_beside_a_relaxed_retirement() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    world.application.with_host_advancement(|phase| {
        let execution = phase.borrowed_request_for_test();
        let host = phase.test_host_request_for_test();
        let custody = host.phase_for_application_for_test(&world.application);
        let before = world.selected_product(&host);
        // principal-0 owns account-1 and account-2 (owner-1, owner-2). Neither
        // activity has an AccountOwner edge, so retiring one leaves no audit-only
        // link in the anchor's adjacency.
        let anchor = before.resolve_entity(
            &custody, PrincipalIdentityField::at("PrincipalIdentityField"), 1, &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        ).unwrap().entity_id_for_test();
        let activity = |key: &str| before.resolve_entity(
            &custody, ActivityIdentity::at("ActivityIdentity"), key.to_owned(), &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        ).unwrap().entity_id_for_test();
        let (retired_entity, unrelated) = (activity("activity-secondary"), activity("activity-primary"));
        let delete = |entity| WorkerIntentBatch::new("adjacency-beside-retirement").push(
            MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: entity }))
        );
        publish_relational_mutation(&execution, &world, delete(retired_entity));
        let original = world.selected_product(&host);
        let graph = (*world.application.runtime_for_test()).primary_graph_for_test().unwrap();
        let relation_kind = *(*graph.layout_for_test())
            .relation(AccountOwner::reference().name()).unwrap().kind_for_test();
        let (retirement, revision, owner_edge) = graph.integration_handle_for_test().with_runtime(&execution, |runtime| {
            let snapshot = original.application_basis_for_test().snapshot_handle();
            let view = runtime.read_truth().project_snapshot(snapshot).unwrap();
            let retired = view.read_entity_retirement(&execution, retired_entity).unwrap().unwrap();
            let retirement = Fact::RetiredOutputEntity {
                entity_id: retired_entity, kind: retired.kind_id(),
                created_at: retired.created_at_version(), deleted_at: retired.deleted_at_version(),
                read_locator: Fact::Entity { entity_id: retired_entity, kind: retired.kind_id() }.locator_identity_for_test(),
            };
            let relations = observe_adjacency(
                runtime, &execution, snapshot, relation_kind, anchor, DecisionDirection::Outgoing, 16,
            ).unwrap().unwrap();
            assert_eq!(relations.len(), 2, "principal-0 owns both accounts");
            let owner_edge = *relations[0].relation_id_for_test();
            let revision = rebase_decision_adjacency(&execution, runtime, snapshot, Fact::Adjacency {
                relation_kind, anchor, direction: DecisionDirection::Outgoing,
                maximum_work_units: 16, relations,
            });
            assert!(matches!(revision, Fact::SourceAdjacencyRevision { native_revision: Some(_), .. }),
                "a live anchor with a nonzero limit acquires its native revision");
            (retirement, revision, owner_edge)
        });
        // Retirement first: a comparison that stops after the first accepted
        // retirement never reaches the adjacency arm.
        let facts: FactPayload = std::sync::Arc::<[Fact]>::from(vec![retirement.clone(), revision]).into();
        publish_relational_mutation(&execution, &world, delete(unrelated));
        let same = world.selected_product(&host);
        publish_relational_mutation(&execution, &world, WorkerIntentBatch::new("move-owner-adjacency").push(
            MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent { relation_id: owner_edge }))
        ));
        let moved = world.selected_product(&host);
        graph.integration_handle_for_test().with_runtime(&execution, |runtime| {
            let original_snapshot = original.application_basis_for_test().snapshot_handle();
            let same_snapshot = same.application_basis_for_test().snapshot_handle();
            let moved_snapshot = moved.application_basis_for_test().snapshot_handle();
            let original_view = runtime.read_truth().project_snapshot(original_snapshot).unwrap();
            let same_view = runtime.read_truth().project_snapshot(same_snapshot).unwrap();
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::product_operation::correction_source::restored_support::overlap::select(execution, &facts, runtime, &original_view, &same_view)
                .expect("retirement and adjacency revision are both admitted support");
            compare(runtime, execution, original_snapshot, same_snapshot, &facts)
                .expect("an unrelated retirement moves neither the retired generation nor the owner adjacency");
            assert!(matches!(
                compare(runtime, execution, original_snapshot, moved_snapshot, &facts),
                Err(Denial::SourceDiverged)
            ), "a removed owner edge diverges the adjacency even though the retirement still holds");
            compare(runtime, execution, original_snapshot, moved_snapshot, &payload(retirement.clone()))
                .expect("the retirement alone still holds at the moved target, so the refusal is the adjacency arm's");
        });
    }).unwrap();
}
