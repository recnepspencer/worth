use worth_query_installation::facade::ApplicationSchema;

#[cfg(feature = "test-primary-graph-faults")]
use worth_relational::facade::{
    identity::PartitionId,
    symbols::ClientKey,
    transactions::{
        AspectFieldPatch, CreateIntent, DeleteRelationIntent, EntityReference, MutationIntent,
        RelationMutationIntent, RelationSpec, WorkerIntentBatch,
    },
};
#[cfg(feature = "test-primary-graph-faults")]
use worth_runtime_world::facade::RuntimeWorldPublicationOutcome;

use super::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Schedules one failure at the generic Query index-publication boundary.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn fail_next_index_publication_for_test(&self) {
        self.primary_provider.fail_next_index_publication_for_test();
    }

    /// Delays one output-readiness delivery before its unique World change is consumed.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn delay_next_output_readiness_delivery_for_test(&self) {
        self.primary_provider
            .delay_next_output_readiness_delivery_for_test();
    }

    /// Fails one readiness evaluation after performed-change delivery.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn fail_next_output_readiness_evaluation_for_test(&self) {
        self.primary_provider
            .fail_next_output_readiness_evaluation_for_test();
    }

    /// Holds real World observations while the next ready output opens a read.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn press_next_ready_read_with_world_snapshots_for_test(&self) {
        self.primary_provider
            .press_next_ready_read_with_world_snapshots_for_test();
    }

    /// Holds real World observations while the next readiness evaluation selects its basis.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn press_next_readiness_with_world_snapshots_for_test(&self) {
        self.primary_provider
            .press_next_readiness_with_world_snapshots_for_test();
    }

    /// Counts readiness decisions actually attempted by this application runtime.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn output_readiness_attempt_count_for_test(&self) -> u64 {
        self.next_output_producer_attempt
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Counts post-publication checkpoints and any World snapshots retained inside their receipts.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn output_checkpoint_snapshot_state_for_test(&self) -> (usize, usize) {
        self.output_demands.output_checkpoint_snapshot_state()
    }

    #[cfg(feature = "test-primary-graph-faults")]
    pub(in crate::domain_computation::primary_graph) fn hold_world_snapshot_pressure_for_test(
        &self,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Vec<crate::basis::WorthQueryProductBranchLease> {
        let mut held = Vec::new();
        let mut exhausted = false;
        for _ in 0..1_024 {
            match self.product_runtime.integration_admit_product_branch(receipt.product_branch()) {
                Ok(observation) => held.push(observation),
                Err(crate::basis::WorthQueryProductBranchAdmissionDenial::ObservationCapacityExhausted) => {
                    exhausted = true;
                    break;
                }
                Err(error) => panic!("World snapshot pressure failed for another reason: {error:?}"),
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
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn prepared_required_output_source_count_for_test(&self) -> usize {
        self.output_demands.prepared_source_count()
    }

    /// Counts retained source custody, including consumed roots awaiting program completion.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn retained_source_custody_count_for_test(&self) -> usize {
        self.output_demands.retained_source_custody_count()
    }

    /// Observes the currently bound primary Bridge truth snapshot.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn primary_truth_snapshot_for_test(
        &self,
    ) -> Option<worth_runtime_bridge::facade::TruthSnapshotIdentity> {
        self.primary_provider
            .graph
            .current_truth_snapshot(&super::super::primary_truth_branch_identity())
    }

    /// Inspects the real Relational snapshot count and current branch basis.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn relational_snapshot_state_for_test(
        &self,
    ) -> (
        usize,
        worth_relational::facade::branch::RelationalBranchBasisDescriptor,
    ) {
        self.primary_provider.graph.with_runtime_mut(|runtime| {
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

    /// Cycles one definition-node membership without changing its visible endpoints.
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn cycle_workflow_definition_membership_for_test(
        &self,
        definition: &crate::domain_computation::primary_graph::PublishedWorkflowDefinitionRef,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) {
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
            runtime
                .prepare_branch_transaction(transaction)
                .expect("the membership cycle prepares a real Relational candidate")
        });
        drop(application_basis);
        let prepared = product
            .publication_binding()
            .prepare_relational_candidate(candidate, request, false)
            .expect("the membership cycle prepares a World publication");
        let outcome = prepared.execute();
        let RuntimeWorldPublicationOutcome::Performed(performed) = outcome else {
            panic!("the membership cycle must perform: {outcome:?}");
        };
        let published_basis = performed.commit().basis().relational_basis().clone();
        drop(performed.consume());
        handle.with_runtime_mut(|runtime| {
            handle
                .ensure_primary_indexes_for_basis(runtime, &published_basis)
                .expect("the membership cycle refreshes indexes at its published basis");
        });
    }

    #[doc(hidden)]
    #[cfg(any(test, feature = "test-durability-faults"))]
    pub fn fail_next_durable_append_for_test(&self) {
        self.primary_provider
            .graph
            .with_runtime_mut(|runtime| runtime.fail_next_durable_append_for_test());
    }

    #[cfg(test)]
    pub(crate) fn provider_session_resource_count(&self) -> usize {
        self.primary_provider.application_attempt_resource_count()
    }

    #[cfg(test)]
    pub(crate) fn application_attempt_work(
        &self,
    ) -> crate::domain_computation::primary_graph::provider::WorthQueryApplicationAttemptWorkSnapshot
    {
        self.primary_provider.application_attempt_work()
    }
}
