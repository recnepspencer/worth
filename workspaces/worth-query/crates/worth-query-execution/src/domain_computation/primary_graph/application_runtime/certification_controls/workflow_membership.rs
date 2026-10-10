//! workflow membership.
use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Attempts to cycle one published definition-node membership natively.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn attempt_workflow_definition_membership_cycle_for_test(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        definition: &crate::domain_computation::primary_graph::PublishedWorkflowDefinitionRef,
    ) -> Result<
        Result<(), worth_relational::facade::transactions::TransactionCommitError>,
        crate::domain_computation::primary_graph::WorthQueryForeignAdvancementPhase,
    > {
        let execution = phase.execution_request_for(&self.product_runtime)?;
        let selected = self
            .on_branch(definition.branch())
            .select(phase)
            .expect("the workflow definition branch remains admitted");
        let (_, product, application_basis) = selected.into_parts();
        let handle = self
            .runtime
            .primary_graph()
            .expect("the application retains its primary graph")
            .integration_handle();
        let candidate = handle.with_query_runtime_mut(&execution, |runtime, layout| {
            let relation_kind = layout.workflow().definition_node_relation;
            let membership = runtime
                .read_truth()
                .visible_relations_of_kind(
                    &execution,
                    relation_kind,
                    application_basis.snapshot_handle().version_id(),
                )
                .into_iter()
                .find(|relation| relation.source == definition.entity_id())
                .expect("the published definition retains a node membership");
            let batch = WorkerIntentBatch::new("workflow-membership-aba")
                .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                    DeleteRelationIntent {
                        relation_id: membership.relation_id,
                    },
                )))
                .push(MutationIntent::Create(CreateIntent::Relation(
                    RelationSpec {
                        partition_id: PartitionId::main(),
                        kind_id: relation_kind,
                        client_key: ClientKey::raw("workflow-membership-aba"),
                        source: EntityReference::Existing(membership.source),
                        target: EntityReference::Existing(membership.target),
                        fields: AspectFieldPatch::default(),
                    },
                )));
            let mut transaction = runtime
                .begin_branch_transaction(
                    product.relational_basis(),
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .expect("the selected product basis admits the fault transaction");
            transaction
                .push_batch(batch, NativeAllocationPolicy::SystemAllocation)
                .expect("the membership cycle stays within resource budgets");
            runtime
                .prepare_branch_transaction(transaction, NativeAllocationPolicy::SystemAllocation)
                .map(|_| ())
        });
        drop(application_basis);
        Ok(candidate)
    }
}
