//! A genuine all-Create publication under the same actual installed owner.
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::Application;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::*;
use worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::publication_denial::WorthQueryProductPublicationOutcome as PublicationOutcome;
use std::collections::BTreeMap;
use worth_query_decl::facade::application_schema::ApplicationScalarValueBinding;
use worth_query_decl::facade::application_schema::StringApplicationValueBinding;
use worth_query_decl::facade::application_schema::U64ApplicationValueBinding;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::mvcc::RequiredOutputPublication;
use worth_relational::facade::mvcc::RequiredOutputPublicationMeaning;
use worth_relational::facade::mvcc::RequiredOutputRolePosture;
use worth_relational::facade::mvcc::RequiredOutputRoleTarget;
use worth_relational::facade::transactions::*;

pub fn admit_genuine_all_create(runtime: &Application) {
    let scope = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    runtime
        .with_application_advancement(&scope, |phase| {
            let execution = phase
                .execution_request_for_for_test((*runtime.runtime_for_test()).product_runtime_for_test())
                .unwrap();
            let (_, product, _application_basis) = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap()
                .into_parts_for_test();
            let graph = (*(*runtime.runtime_for_test()).primary_provider_for_test()).graph_for_test();
            let publisher = graph.application_publication.as_ref().unwrap();
            let target = CreatedEntityRef {
                partition_id: PartitionId::main(),
                kind_id: (*graph.layout_for_test()).entity_kind_for_test("EntrySet").unwrap(),
                client_key: worth_relational::facade::symbols::ClientKey::raw(
                    "genuine-owner-birth",
                ),
            };
            let producer = "native-owner-oracle-all-create";
            let dependency = [41; 32];
            // The route's source is a real entity from the already completed Query.
            let anchor = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap()
                .resolve_entity(
                    &phase,
                    BodyKey::reference(),
                    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::ANCHOR.to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let meaning = RequiredOutputPublicationMeaning {
                implementation: worth_relational::facade::mvcc::RequiredOutputProducingImplementation::from_checkpoint([0x11; 16], [0x22; 32]).unwrap(),
                family: producer.to_owned(),
                source_scope: anchor.entity_id_for_test(),
                source_partition: worth_foundational::CanonicalDigestId::new([42; 32]),
                source_epoch: worth_foundational::CanonicalDigestId::new([43; 32]),
            };
            let roles = vec![RequiredOutputRoleTarget {
                role: "all-created".into(),
                entity_name: "EntrySet".into(),
                kind: target.kind_id,
                posture: RequiredOutputRolePosture::Create,
                target: EntityReference::Created(target.clone()),
            }];
            let candidate = graph.with_query_runtime_for_test(&execution, |native, layout| {
                let fields = AspectFieldPatch::from(BTreeMap::from([
                    (
                        layout
                            .field_locator_for_test("EntrySet", "EntrySetFacts", "EntrySetKey")
                            .unwrap()
                            .clone(),
                        StringApplicationValueBinding::encode(&"genuine-owner-birth".to_owned())
                            .unwrap(),
                    ),
                    (
                        layout
                            .field_locator_for_test("EntrySet", "EntrySetFacts", "EntrySetWeight")
                            .unwrap()
                            .clone(),
                        U64ApplicationValueBinding::encode(&9).unwrap(),
                    ),
                ]));
                let batch = WorkerIntentBatch::new(producer).push(MutationIntent::Create(
                    CreateIntent::Entity(EntitySpec {
                        partition_id: target.partition_id,
                        kind_id: target.kind_id,
                        client_key: target.client_key.clone(),
                        fields,
                    }),
                ));
                let absence = native
                    .readmit_required_output_route_absence(
                        &execution,
                        &publisher.native,
                        product.relational_basis_for_test(),
                        &meaning,
                    )
                    .unwrap()
                    .expect("new genuine route");
                let mut transaction = native
                    .begin_branch_transaction(
                        product.relational_basis_for_test(),
                        RelationalTransactionIntent::ordinary(),
                    )
                    .unwrap();
                transaction
                    .push_batch(batch, execution.allocation_policy())
                    .unwrap();
                let proposal = transaction
                    .validate(&execution, native)
                    .unwrap()
                    .bind_required_output_publication(
                        &publisher.native,
                        RequiredOutputPublication::Birth {
                            admission: (*publisher.owner_for_test()).admit_required_output(producer, dependency),
                            absence: Some(absence),
                            meaning: meaning.clone(),
                            roles,
                        },
                    )
                    .unwrap();
                native
                    .prepare_validated_proposal(&execution, proposal)
                    .unwrap()
            });
            let outcome = product
                .publication_binding_for_test()
                .prepare_relational_candidate_for_test(candidate, &scope, false)
                .unwrap()
                .execute_for_test(execution);
            let Ok(PublicationOutcome::Performed(performed)) = outcome else {
                panic!("genuine all-Create publication must perform: {outcome:?}")
            };
            let id = performed
                .component_results()
                .relational_commit_result()
                .unwrap()
                .created_entity(&target)
                .unwrap();
            let basis = performed.commit().basis().relational_basis().clone();
            let roles = vec![RequiredOutputRoleTarget {
                role: "all-created".into(),
                entity_name: "EntrySet".into(),
                kind: target.kind_id,
                posture: RequiredOutputRolePosture::Create,
                target: EntityReference::Existing(id),
            }];
            drop(performed.consume());
            graph.with_runtime(&execution, |native| {
                native
                    .readmit_required_output_publication(
                        &execution,
                        &publisher.native,
                        &basis,
                        producer,
                        &dependency,
                        &meaning,
                        &roles,
                    )
                    .expect("genuine all-Create has complete native ownership");
            });
        })
        .unwrap();
}
