//! Native probes for published workflow node custody.
use std::collections::BTreeMap;
use worth_foundational::facade::{AspectValue, InternedString};

use worth_relational::facade::{
    identity::PartitionId,
    symbols::ClientKey,
    transactions::{
        AspectFieldPatch, CreateIntent, CreatedEntityRef, DeleteRelationIntent,
        EntityMutationIntent, EntityReference, EntitySpec, MutationIntent, RelationMutationIntent,
        RelationSpec, UpdateEntityFieldsIntent, WorkerIntentBatch,
    },
};

use super::{ApplicationSchema, WorthQueryPrimaryGraphApplicationRuntime};

impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Attempts to cycle one published definition-node membership natively.
    #[doc(hidden)]
    pub fn attempt_workflow_definition_membership_cycle_for_test(
        &self,
        definition: &crate::domain_computation::primary_graph::PublishedWorkflowDefinitionRef,
    ) -> Result<
        Result<(), worth_relational::facade::transactions::TransactionCommitError>,
        crate::facade::primary_graph::WorthQueryHandleDenial,
    > {
        self.primary_provider.graph.with_runtime(|_| ())?;
        let selected = self
            .on_branch(definition.branch())
            .select()
            .expect("the workflow definition branch remains admitted");
        let (_, product, application_basis) = selected.into_parts();
        let handle = self
            .runtime
            .primary_graph()
            .expect("the application retains its primary graph")
            .integration_handle();
        let candidate = handle.with_query_runtime_mut(|runtime, layout| {
            let relation_kind = layout.workflow().definition_node_relation;
            let membership = runtime
                .read_truth()
                .visible_relations_of_kind(
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
                .push_batch(batch)
                .expect("the membership cycle stays within resource budgets");
            runtime.prepare_branch_transaction(transaction).map(|_| ())
        });
        drop(application_basis);
        candidate
    }

    /// Attempts to alter one published node's meaning without touching membership.
    #[doc(hidden)]
    pub fn attempt_workflow_node_field_update_for_test(
        &self,
        definition: &crate::domain_computation::primary_graph::PublishedWorkflowDefinitionRef,
    ) -> Result<
        Result<(), worth_relational::facade::transactions::TransactionCommitError>,
        crate::facade::primary_graph::WorthQueryHandleDenial,
    > {
        self.primary_provider.graph.with_runtime(|_| ())?;
        let selected = self
            .on_branch(definition.branch())
            .select()
            .expect("the workflow definition branch remains admitted");
        let (_, product, application_basis) = selected.into_parts();
        let handle = self
            .runtime
            .primary_graph()
            .expect("the application retains its primary graph")
            .integration_handle();
        let candidate = handle.with_query_runtime_mut(|runtime, layout| {
            let membership = runtime
                .read_truth()
                .visible_relations_of_kind(
                    layout.workflow().definition_node_relation,
                    application_basis.snapshot_handle().version_id(),
                )
                .into_iter()
                .find(|relation| relation.source == definition.entity_id())
                .expect("the published definition retains a node membership");
            let fields = AspectFieldPatch::from(BTreeMap::from([(
                layout.workflow().node.path.clone(),
                AspectValue::String(InternedString::Raw("tampered-node".to_owned())),
            )]));
            let batch =
                WorkerIntentBatch::new("workflow-node-field-update").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: membership.target,
                        fields,
                    }),
                ));
            let mut transaction = runtime
                .begin_branch_transaction(
                    product.relational_basis(),
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .expect("the selected product basis admits the fault transaction");
            transaction
                .push_batch(batch)
                .expect("the field update stays within resource budgets");
            runtime.prepare_branch_transaction(transaction).map(|_| ())
        });
        drop(application_basis);
        candidate
    }

    /// Attempts to attach a newly created definition to a published node.
    #[doc(hidden)]
    pub fn attempt_workflow_existing_node_attachment_for_test(
        &self,
        definition: &crate::domain_computation::primary_graph::PublishedWorkflowDefinitionRef,
    ) -> Result<
        Result<(), worth_relational::facade::transactions::TransactionCommitError>,
        crate::facade::primary_graph::WorthQueryHandleDenial,
    > {
        self.primary_provider.graph.with_runtime(|_| ())?;
        let selected = self
            .on_branch(definition.branch())
            .select()
            .expect("the workflow definition branch remains admitted");
        let (_, product, application_basis) = selected.into_parts();
        let handle = self
            .runtime
            .primary_graph()
            .expect("the application retains its primary graph")
            .integration_handle();
        let candidate = handle.with_query_runtime_mut(|runtime, layout| {
            let workflow = layout.workflow();
            let membership = runtime
                .read_truth()
                .visible_relations_of_kind(
                    workflow.definition_node_relation,
                    application_basis.snapshot_handle().version_id(),
                )
                .into_iter()
                .find(|relation| relation.source == definition.entity_id())
                .expect("the published definition retains a node membership");
            let source = CreatedEntityRef {
                partition_id: membership.target.partition_id,
                kind_id: workflow.definition.entity_kind,
                client_key: ClientKey::raw("workflow-new-definition-attachment"),
            };
            let batch = WorkerIntentBatch::new("workflow-existing-node-attachment")
                .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: source.partition_id,
                    kind_id: source.kind_id,
                    client_key: source.client_key.clone(),
                    fields: AspectFieldPatch::default(),
                })))
                .push(MutationIntent::Create(CreateIntent::Relation(
                    RelationSpec {
                        partition_id: membership.relation_id.partition_id,
                        kind_id: workflow.definition_node_relation,
                        client_key: ClientKey::raw("workflow-existing-node-attachment"),
                        source: EntityReference::Created(source),
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
                .push_batch(batch)
                .expect("the attachment stays within resource budgets");
            runtime.prepare_branch_transaction(transaction).map(|_| ())
        });
        drop(application_basis);
        candidate
    }
}
