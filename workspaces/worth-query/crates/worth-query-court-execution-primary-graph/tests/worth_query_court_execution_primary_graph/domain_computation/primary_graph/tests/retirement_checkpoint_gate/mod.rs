//! Fresh native admission cannot turn descriptive tombstones into producer provenance.
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::restored_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as WorthQueryApplicationOutputPosture;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryCheckpointOutputRole;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::native_output_witness::SealedNativeOutputWitness;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact as Fact;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondence;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;
use worth_execution::ExecutionAllocationPolicy as CapturePolicy;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::transactions::DeleteEntityIntent;
use worth_relational::facade::transactions::EntityMutationIntent;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;

#[test]
fn fresh_native_retirement_descriptions_do_not_issue_original_output_authority() {
    {
        let source = installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
        let (original, unrelated) = source
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                let selected = source.selected_product(&host_custody);
                let resolve = |key: &str| {
                    selected
                        .resolve_entity(
                            &host_custody.phase_for_application_for_test(&source.application),
                            AccountIdentity::reference(),
                            key.to_owned(),
                            &live_scope(),
                            WorthQueryPrincipalResolutionMode::Ordinary,
                        )
                        .unwrap()
                        .entity_id_for_test()
                };
                let original = resolve("account-2");
                let unrelated = resolve("account-1");
                drop(selected);

                (original, unrelated)
            })
            .expect("source read owns its host call");
        for entity in [original, unrelated] {
            source
                .application
                .with_host_advancement(|phase| {
                    let host_custody = phase.test_host_request_for_test();

                    publish_relational_mutation(
                        &host_custody.borrowed_request_for_test(),
                        &source,
                        WorkerIntentBatch::new("checkpoint-retirement-gate").push(
                            MutationIntent::Entity(EntityMutationIntent::Delete(
                                DeleteEntityIntent { entity_id: entity },
                            )),
                        ),
                    )
                })
                .expect("the fixture read owns its installed host call");
        }
        let bytes = source
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();

                source
                    .application
                    .capture_application_checkpoint(
                        &host_custody.phase_for_application_for_test(&source.application),
                        CapturePolicy::SystemAllocation,
                    )
                    .unwrap()
            })
            .expect("the fixture read owns its installed host call");
        drop(source);
        let fresh =
            restored_world(bytes).expect("actual opaque native checkpoint freshly installs");
        fresh.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request_for_test(); let borrowed_execution = phase.borrowed_request_for_test(); let execution = &borrowed_execution;        let selected = fresh.selected_product(&host_custody);
        let graph = (*fresh.application.runtime_for_test()).primary_graph_for_test().unwrap();
        let handle = graph.integration_handle_for_test();
        handle.with_runtime(&borrowed_execution, |runtime| {
        let snapshot = selected.application_basis_for_test().snapshot_handle();
        let read = runtime.read_truth();
        let view = read.project_snapshot(snapshot).unwrap();
        let original_metadata = view.read_entity_retirement(execution, original).unwrap().unwrap();
        let unrelated_metadata = view.read_entity_retirement(execution, unrelated).unwrap().unwrap();
        assert_eq!(original_metadata.kind_id(), unrelated_metadata.kind_id());
        assert_ne!(original_metadata.deleted_at_version(), unrelated_metadata.deleted_at_version());
        for entity in [original, unrelated] {
            let record = view.read_entity_retirement(execution, entity).unwrap().unwrap();
            let facts = [Fact::RetiredOutputEntity {
                entity_id: entity, kind: record.kind_id(), created_at: record.created_at_version(),
                deleted_at: record.deleted_at_version(), read_locator: "output-role:7:Account:7:retired".into(),
            }];
            assert!(facts[0].remains_equal_in_for_test(runtime,execution,  snapshot).unwrap(), "both actual tombstones match native truth");
            let correspondence = correspondence(entity, execution);
let candidate = SealedNativeOutputWitness::from_checkpoint_facts_for_test(&correspondence, graph.layout_for_test(), &facts).unwrap();



            let current = candidate.as_ref().is_some_and(|candidate| candidate.get().unwrap()
                .checkpoint_facts_current_in_for_test(execution, runtime, snapshot, &facts).unwrap());
            eprintln!("fresh installed retirement candidate entity={entity:?}, original={}, current={current}", entity == original);
            assert!(candidate.is_none() && !current, "native tombstone descriptions do not bind the original producer retirement");
        }
    });}).expect("restored read owns its host call");
    }
}
pub fn correspondence(
    entity: EntityId,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryApplicationOutputCorrespondence {
    WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles_for_test(
        TypeId::of_for_test::<()>(),
        TypeId::of_for_test::<()>(),
        std::iter::empty(),
        &[WorthQueryCheckpointOutputRole::new_for_test(
            "retired".into(),
            WorthQueryApplicationOutputPosture::Retire,
            "Account".into(),
            entity,
        )],
        |_| Some(TypeId::of_for_test::<()>()),
        execution,
    )
    .unwrap()
}
