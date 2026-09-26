//! Native fault probe for already-issued approval custody.

use std::collections::BTreeMap;

use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::{
    identity::EntityId,
    transactions::{
        AspectFieldPatch, DeleteRelationIntent, EntityMutationIntent, MutationIntent,
        RelationMutationIntent, TransactionCommitError, UpdateEntityFieldsIntent,
        WorkerIntentBatch,
    },
};

use super::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Attempts to change a published approval decision through the native writer.
    #[doc(hidden)]
    pub fn attempt_workflow_approval_field_update_for_test(
        &self,
        instance: &crate::domain_computation::primary_graph::PublishedWorkflowInstanceRef,
        approval: EntityId,
    ) -> Result<(), TransactionCommitError> {
        let selected = self
            .on_branch(instance.branch())
            .select()
            .expect("the workflow instance branch remains admitted");
        let (_, product, application_basis) = selected.into_parts();
        let handle = self
            .runtime
            .primary_graph()
            .expect("the application retains its primary graph")
            .integration_handle();
        let candidate = handle.with_query_runtime_mut(|runtime, layout| {
            let fields = AspectFieldPatch::from(BTreeMap::from([(
                layout.workflow().approval.decision.clone(),
                AspectValue::String(InternedString::Raw("reject".to_owned())),
            )]));
            let batch = WorkerIntentBatch::new("workflow-approval-field-update").push(
                MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                    UpdateEntityFieldsIntent {
                        entity_id: approval,
                        fields,
                    },
                )),
            );
            let mut transaction = runtime
                .begin_branch_transaction(
                    product.relational_basis(),
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .expect("the selected product basis admits the fault transaction");
            transaction
                .push_batch(batch)
                .expect("the approval update stays within resource budgets");
            runtime.prepare_branch_transaction(transaction).map(|_| ())
        });
        drop(application_basis);
        candidate
    }

    /// Attempts to remove one link from an issued approval to its evidence.
    #[doc(hidden)]
    pub fn attempt_workflow_approval_evidence_delete_for_test(
        &self,
        instance: &crate::domain_computation::primary_graph::PublishedWorkflowInstanceRef,
        approval: EntityId,
    ) -> Result<(), TransactionCommitError> {
        let selected = self
            .on_branch(instance.branch())
            .select()
            .expect("the workflow instance branch remains admitted");
        let (_, product, application_basis) = selected.into_parts();
        let handle = self
            .runtime
            .primary_graph()
            .expect("the application retains its primary graph")
            .integration_handle();
        let candidate = handle.with_query_runtime_mut(|runtime, layout| {
            let relation = runtime
                .read_truth()
                .visible_relations_of_kind(
                    layout.workflow().approval_evidence_relation,
                    application_basis.snapshot_handle().version_id(),
                )
                .into_iter()
                .find(|relation| relation.source == approval)
                .expect("the published approval retains an evidence link");
            let batch = WorkerIntentBatch::new("workflow-approval-evidence-delete").push(
                MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
                    relation_id: relation.relation_id,
                })),
            );
            let mut transaction = runtime
                .begin_branch_transaction(
                    product.relational_basis(),
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .expect("the selected product basis admits the fault transaction");
            transaction
                .push_batch(batch)
                .expect("the approval link deletion stays within resource budgets");
            runtime.prepare_branch_transaction(transaction).map(|_| ())
        });
        drop(application_basis);
        candidate
    }
}
