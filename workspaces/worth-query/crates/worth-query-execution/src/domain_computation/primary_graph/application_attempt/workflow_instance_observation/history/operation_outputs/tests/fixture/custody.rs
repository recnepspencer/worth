//! Data valid under the schema must still fail at immutable-history custody.
use super::*;

impl Context {
    pub(in super::super) fn assert_protected_context(&self) {
        let host = payer();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let graph = self.world.application.runtime.primary_graph().unwrap();
        let layout = graph.layout.workflow();
        let edge = self.read(|execution, runtime, snapshot, layout| {
            let entry = observe_adjacency(
                runtime,
                execution,
                snapshot,
                layout.operation_output.membership,
                self.transition,
                Direction::Outgoing,
                8,
            )
            .unwrap()
            .unwrap()[0]
                .to;
            observe_adjacency(
                runtime,
                execution,
                snapshot,
                layout.operation_output.target,
                entry,
                Direction::Outgoing,
                4,
            )
            .unwrap()
            .unwrap()
            .remove(0)
        });
        let alternate = if edge.to == self.created {
            self.source
        } else {
            self.created
        };
        let mutations = [
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                UpdateEntityFieldsIntent {
                    entity_id: edge.from,
                    fields: AspectFieldPatch::from(BTreeMap::from([(
                        layout.operation_output.role.clone(),
                        text_value("rewritten"),
                    )])),
                },
            )),
            MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
                UpdateRelationEndpointsIntent {
                    relation_id: edge.relation_id,
                    kind_id: layout.operation_output.target,
                    source: EntityReference::Existing(edge.from),
                    target: EntityReference::Existing(alternate),
                },
            )),
            MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
                relation_id: edge.relation_id,
            })),
        ];
        for (index, mutation) in mutations.into_iter().enumerate() {
            let (_, product, _lease) = self
                .world
                .application
                .select_product_branch(
                    &execution,
                    self.world.application.product_runtime().default_branch(),
                )
                .unwrap()
                .into_parts();
            let candidate = graph
                .integration_handle()
                .with_runtime_mut(&execution, |runtime| {
                    let mut transaction = runtime
                        .begin_branch_transaction(
                            product.relational_basis(),
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
