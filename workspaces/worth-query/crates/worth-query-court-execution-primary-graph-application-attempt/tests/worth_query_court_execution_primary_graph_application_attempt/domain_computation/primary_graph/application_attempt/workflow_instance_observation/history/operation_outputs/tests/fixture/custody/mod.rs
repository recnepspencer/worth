//! Data valid under the schema must still fail at immutable-history custody.
use crate::domain_computation::primary_graph::application_attempt::workflow_instance_observation::history::operation_outputs::tests::fixture::*;

impl Context {
    pub fn assert_protected_context(&self) {
        let host = payer();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let graph = (*self.world.application.runtime_for_test())
            .primary_graph()
            .unwrap();
        let layout = (*graph.layout_for_test()).workflow();
        let edge = self.read(|execution, runtime, snapshot, layout| {
            let entry = (*observe_adjacency(
                runtime,
                execution,
                snapshot,
                (*layout.operation_output.membership_for_test()),
                self.transition,
                Direction::Outgoing,
                8,
            )
            .unwrap()
            .unwrap()[0]
                .to_for_test());
            observe_adjacency(
                runtime,
                execution,
                snapshot,
                (*layout.operation_output.target_for_test()),
                entry,
                Direction::Outgoing,
                4,
            )
            .unwrap()
            .unwrap()
            .remove(0)
        });
        let alternate = if (*edge.to_for_test()) == self.created {
            self.source
        } else {
            self.created
        };
        let mutations = [
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                UpdateEntityFieldsIntent {
                    entity_id: (*edge.from_for_test()),
                    fields: AspectFieldPatch::from(BTreeMap::from([(
                        (*layout.operation_output.role_for_test()).clone(),
                        text_value("rewritten"),
                    )])),
                },
            )),
            MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
                UpdateRelationEndpointsIntent {
                    relation_id: (*edge.relation_id_for_test()),
                    kind_id: (*layout.operation_output.target_for_test()),
                    source: EntityReference::Existing((*edge.from_for_test())),
                    target: EntityReference::Existing(alternate),
                },
            )),
            MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
                relation_id: (*edge.relation_id_for_test()),
            })),
        ];
        for (index, mutation) in mutations.into_iter().enumerate() {
            let (_, product, _lease) = self
                .world
                .application
                .select_product_branch_for_test(
                    &execution,
                    self.world
                        .application
                        .product_runtime()
                        .default_branch_for_test(),
                )
                .unwrap()
                .into_parts_for_test();
            let candidate = graph
                .integration_handle_for_test()
                .with_runtime_mut_for_test(&execution, |runtime| {
                    let mut transaction = runtime
                        .begin_branch_transaction(
                            product.relational_basis_for_test(),
                            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                        )
                        .unwrap();
                    transaction
                        .push_batch(
                            WorkerIntentBatch::new("rewrite-protected-output").push(mutation),
                            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                        )
                        .unwrap();
                    runtime.prepare_branch_transaction(&execution, transaction)
                });
            let Err(TransactionCommitError::Conflict { error, .. }) = candidate else {
                panic!("protected output change {index} passed preparation")
            };
            assert!(
                matches!(error.class, ConflictClass::InvariantViolation { .. }),
                "{error:?}"
            );
            if index < 2 {
                assert!(
                    error
                        .detail
                        .contains("worth-query.workflow.publication-immutability"),
                    "{error:?}"
                );
            }
        }
        self.check().unwrap();
    }
}
