//! A genuine all-Create publication under the same actual installed owner.
use super::installation::Application;
use super::*;
use crate::domain_computation::execution_runtime::product_world::publication_denial::WorthQueryProductPublicationOutcome as PublicationOutcome;
use std::collections::BTreeMap;
use worth_query_decl::facade::application_schema::{
    ApplicationScalarValueBinding, StringApplicationValueBinding, U64ApplicationValueBinding,
};
use worth_relational::facade::{
    identity::PartitionId,
    mvcc::{
        RelationalTransactionIntent, RequiredOutputPublication, RequiredOutputPublicationMeaning,
        RequiredOutputRolePosture, RequiredOutputRoleTarget,
    },
    transactions::*,
};

pub(super) fn admit_genuine_all_create(runtime: &Application) {
    let scope = primary_graph::tests::fixture::live_scope();
    runtime
        .with_application_advancement(&scope, |phase| {
            let execution = phase
                .execution_request_for(&runtime.product_runtime)
                .unwrap();
            let (_, product, _application_basis) = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap()
                .into_parts();
            let graph = &runtime.primary_provider.graph;
            let publisher = graph.application_publication.as_ref().unwrap();
            let target = CreatedEntityRef {
                partition_id: PartitionId::main(),
                kind_id: graph.layout.entity_kind("EntrySet").unwrap(),
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
                    installation::ANCHOR.to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let meaning = RequiredOutputPublicationMeaning {
                implementation: worth_relational::facade::mvcc::RequiredOutputProducingImplementation::from_checkpoint([0x11; 16], [0x22; 32]).unwrap(),
                family: producer.to_owned(),
                source_scope: anchor.entity_id(),
                source_partition: [42; 32],
                source_epoch: [43; 32],
            };
            let roles = vec![RequiredOutputRoleTarget {
                role: "all-created".into(),
                entity_name: "EntrySet".into(),
                kind: target.kind_id,
                posture: RequiredOutputRolePosture::Create,
                target: EntityReference::Created(target.clone()),
            }];
            let candidate = graph.with_query_runtime_mut(&execution, |native, layout| {
                let fields = AspectFieldPatch::from(BTreeMap::from([
                    (
                        layout
                            .field_locator("EntrySet", "EntrySetFacts", "EntrySetKey")
                            .unwrap()
                            .clone(),
                        StringApplicationValueBinding::encode(&"genuine-owner-birth".to_owned())
                            .unwrap(),
                    ),
                    (
                        layout
                            .field_locator("EntrySet", "EntrySetFacts", "EntrySetWeight")
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
                        product.relational_basis(),
                        &meaning,
                    )
                    .unwrap()
                    .expect("new genuine route");
                let mut transaction = native
                    .begin_branch_transaction(
                        product.relational_basis(),
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
                            admission: publisher.owner.admit_required_output(producer, dependency),
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
                .publication_binding()
                .prepare_relational_candidate(candidate, &scope, false)
                .unwrap()
                .execute(execution);
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
