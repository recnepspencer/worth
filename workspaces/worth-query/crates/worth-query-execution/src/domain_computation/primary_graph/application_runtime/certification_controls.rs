mod actor_retention;
mod correction_history_indexes;
mod managed_cleanup;
use crate::basis::WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal;
#[path = "certification_controls/workflow_membership.rs"]
mod workflow_membership;
use worth_execution::ExecutionAllocationPolicy as NativeAllocationPolicy;
use worth_query_installation::facade::ApplicationSchema;

use std::collections::BTreeMap;
use worth_foundational::facade::{AspectValue, InternedString};

use super::WorthQueryPrimaryGraphApplicationRuntime;
mod native_field_write;
mod publication;
mod workflow_approval;
mod workflow_definition;
use worth_relational::facade::{
    identity::PartitionId,
    symbols::ClientKey,
    transactions::{
        AspectFieldPatch, CreateIntent, CreatedEntityRef, DeleteRelationIntent,
        EntityMutationIntent, EntityReference, EntitySpec, MutationIntent, RelationMutationIntent,
        RelationSpec, UpdateEntityFieldsIntent, WorkerIntentBatch,
    },
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Schedules failure at the real post-commit snapshot admission boundary.
    #[doc(hidden)]
    pub fn fail_next_post_commit_snapshot_for_test(&self) {
        self.primary_provider
            .fail_next_post_commit_snapshot_for_test();
    }

    /// Drops only rebuildable workflow-instance progress projections.
    #[doc(hidden)]
    pub fn release_workflow_instance_progress_for_test(&self) {
        self.runtime
            .retain_primary_graph_integration_handle()
            .expect("a published application runtime retains its primary graph")
            .release_workflow_instance_progress();
    }

    /// Evicts every compiled workflow definition, leaving revision pins intact.
    #[doc(hidden)]
    pub fn release_workflow_compilation_for_test(&self) {
        self.runtime
            .retain_primary_graph_integration_handle()
            .expect("a published application runtime retains its primary graph")
            .with_workflow_compilation_reuse_mut(|reuse| reuse.release_all());
    }

    /// Delays one output-readiness delivery before its unique World change is consumed.
    #[doc(hidden)]
    pub fn delay_next_output_readiness_delivery_for_test(&self) {
        self.primary_provider
            .delay_next_output_readiness_delivery_for_test();
    }

    /// Fails one readiness evaluation after performed-change delivery.
    #[doc(hidden)]
    pub fn fail_next_output_readiness_evaluation_for_test(&self) {
        self.primary_provider
            .fail_next_output_readiness_evaluation_for_test();
    }

    /// Holds real World observations while the next ready output opens a read.
    #[doc(hidden)]
    pub fn press_next_ready_read_with_world_snapshots_for_test(&self) {
        self.primary_provider
            .press_next_ready_read_with_world_snapshots_for_test();
    }

    /// Holds real World observations while the next readiness evaluation selects its basis.
    #[doc(hidden)]
    pub fn press_next_readiness_with_world_snapshots_for_test(&self) {
        self.primary_provider
            .press_next_readiness_with_world_snapshots_for_test();
    }

    /// Counts readiness decisions actually attempted by this application runtime.
    #[doc(hidden)]
    pub fn output_readiness_attempt_count_for_test(&self) -> u64 {
        self.next_output_producer_attempt
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Counts post-publication checkpoints and any World snapshots retained inside their receipts.
    #[doc(hidden)]
    pub fn output_checkpoint_snapshot_state_for_test(&self) -> (usize, usize) {
        self.output_demands.output_checkpoint_snapshot_state()
    }

    pub(in crate::domain_computation::primary_graph) fn hold_world_snapshot_pressure_for_test(
        &self,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Vec<crate::basis::WorthQueryProductBranchLease> {
        let mut held = Vec::new();
        let mut exhausted = false;
        for _ in 0..1_024 {
            match self
                .product_runtime
                .integration_admit_product_branch(branch)
            {
                Ok(observation) => held.push(observation),
                Err(crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(
                    AdmissionRefusal::ObservationCapacityExhausted,
                )) => {
                    exhausted = true;
                    break;
                }
                Err(error) => {
                    panic!("World snapshot pressure failed for another reason: {error:?}")
                }
            }
        }
        assert!(
            exhausted,
            "snapshot pressure must reach actual World capacity"
        );
        held
    }

    /// Counts performed sources awaiting their required-output admission owner.
    #[doc(hidden)]
    pub fn prepared_required_output_source_count_for_test(&self) -> usize {
        self.output_demands.prepared_source_count()
    }

    /// Counts retained source custody, including consumed roots awaiting program completion.
    #[doc(hidden)]
    pub fn retained_source_custody_count_for_test(&self) -> usize {
        self.output_demands.retained_source_custody_count()
    }

    /// Observes the currently bound primary Bridge truth snapshot.
    #[doc(hidden)]
    pub fn primary_truth_snapshot_for_test(
        &self,
    ) -> Option<worth_runtime_bridge::facade::TruthSnapshotIdentity> {
        self.primary_provider
            .graph
            .current_truth_snapshot(&super::super::primary_truth_branch_identity())
    }

    /// Inspects the real Relational snapshot count and current branch basis.
    #[doc(hidden)]
    pub fn relational_snapshot_state_for_test(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> (
        usize,
        worth_relational::facade::branch::RelationalBranchBasisDescriptor,
    ) {
        self.primary_provider
            .graph
            .with_runtime_mut(request_execution, |runtime| {
                let active = runtime.retention().inspect_plan().active_snapshot_count;
                let identity = runtime
                    .branch_identity(self.relational_branch_identity.branch_id())
                    .expect("the application branch remains owner registered");
                let (_, basis) = runtime
                    .observe_branch(&identity)
                    .expect("the application branch remains owner observable");
                (active, basis.descriptor().clone())
            })
    }

    /// Attempts to alter one published node's meaning without touching membership.
    #[doc(hidden)]
    pub fn attempt_workflow_node_field_update_for_test(
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
            let membership = runtime
                .read_truth()
                .visible_relations_of_kind(
                    &execution,
                    layout.workflow().definition_node_relation,
                    application_basis.snapshot_handle().version_id(),
                )
                .expect("certification fixture read request remains live")
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
                .push_batch(batch, NativeAllocationPolicy::SystemAllocation)
                .expect("the field update stays within resource budgets");
            runtime
                .prepare_branch_transaction(&execution, transaction)
                .map(|_| ())
        });
        drop(application_basis);
        Ok(candidate)
    }

    /// Attempts to attach a newly created definition to a published node.
    #[doc(hidden)]
    pub fn attempt_workflow_existing_node_attachment_for_test(
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
            let workflow = layout.workflow();
            let membership = runtime
                .read_truth()
                .visible_relations_of_kind(
                    &execution,
                    workflow.definition_node_relation,
                    application_basis.snapshot_handle().version_id(),
                )
                .expect("certification fixture read request remains live")
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
                .push_batch(batch, NativeAllocationPolicy::SystemAllocation)
                .expect("the attachment stays within resource budgets");
            runtime
                .prepare_branch_transaction(&execution, transaction)
                .map(|_| ())
        });
        drop(application_basis);
        Ok(candidate)
    }

    #[doc(hidden)]
    pub fn fail_next_durable_append_for_test(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) {
        self.primary_provider
            .graph
            .with_runtime_mut(request_execution, |runtime| {
                runtime.fail_next_durable_append_for_test()
            });
    }

    #[cfg(test)]
    pub(crate) fn provider_session_resource_count_for_test(&self) -> usize {
        self.primary_provider.application_attempt_resource_count()
    }

    #[cfg(test)]
    pub(crate) fn application_attempt_work_for_test(
        &self,
    ) -> crate::domain_computation::primary_graph::provider::WorthQueryApplicationAttemptWorkSnapshot
    {
        self.primary_provider.application_attempt_work_for_test()
    }
}

mod primary_index_generations;
