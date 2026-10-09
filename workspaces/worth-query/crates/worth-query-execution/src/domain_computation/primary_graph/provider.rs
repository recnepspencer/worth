mod aftermath_causality;
pub(in crate::domain_computation::primary_graph) mod relational_execution_denial;
pub(in crate::domain_computation) use aftermath_causality::WorthQueryAftermathCausalityReadDenial;
mod application_attempt_state;
pub(in crate::domain_computation::primary_graph::provider) use application_attempt_state::publish_recovered;
pub(in crate::domain_computation::primary_graph::provider) use application_attempt_state::ManagedUnpublishedAttempt;
pub(in crate::domain_computation::primary_graph) use application_attempt_state::{
    FactlessCurrentness, OwnEffectOnReads, RebaseVerificationReason,
};
mod application_attempt_work;
mod application_decision_fact;
mod application_touch_admission;
mod branch_commit_coordination;
mod commit_causality;
pub(super) mod committed_dispatch_outbox;
mod completed_observation;
mod decision_facts;
pub(in crate::domain_computation::primary_graph) mod dispatch_outbox;
mod evidence_window;
pub(in crate::domain_computation::primary_graph) mod fault_port;
mod graph_participation;
mod history_retirement;
mod idempotency;
mod inbound_completion;
mod inbound_cost;
pub(in crate::domain_computation::primary_graph) use inbound_completion::WorthQueryInboundCompletionPreparationDenial;
pub(in crate::domain_computation::primary_graph) use inbound_completion::{
    WorthQueryCanonicalCompletionRow, WorthQueryInboundCompletionReadDenial,
};
mod inbound_terminal_index;
pub(in crate::domain_computation) use inbound_terminal_index::{
    WorthQueryCanonicalInboundCompletion, WorthQueryInboundTerminalIndexDenial,
};
mod installation;
mod invariant_execution;
pub(in crate::domain_computation::primary_graph) mod invariant_execution_failure;
mod managed_application_recovery;
mod mutation_work;
pub use managed_application_recovery::{
    WorthQueryManagedApplicationRecoveryDenial, WorthQueryManagedApplicationRecoveryOutcome,
    WorthQueryManagedApplicationRecoveryPerformed,
};
mod output_readiness_fault;
mod outstanding_dispatch;
pub(in crate::domain_computation) use outstanding_dispatch::WorthQueryTerminalDispatchReleaseDenial;
pub(in crate::domain_computation::primary_graph) use outstanding_dispatch::{
    OutstandingDispatchInFlightLease, WorthQueryOutstandingInFlightDenial,
};
mod pending_application_publication;
pub(in crate::domain_computation::primary_graph) use outstanding_dispatch::OutstandingDispatchReservation;
pub(in crate::domain_computation::primary_graph) use pending_application_publication::WorthQueryApplicationPublicationRecoveryReservation;
mod product_retirement;
mod provisional_state;
mod publication_recovery;
mod resource_support;
pub(in crate::domain_computation::primary_graph) use resource_support::WorthQueryPrimaryGraphResourceSupport;
mod session_commit;
mod session_lifecycle;
mod terminal_dispatch_release;
mod unpublished_idempotency;
#[cfg(all(test, feature = "test-world-operation-control"))]
pub(in crate::domain_computation::primary_graph) use unpublished_idempotency::unwind_recovery_inspection_count;
pub(in crate::domain_computation) use unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition;
pub(in crate::domain_computation::primary_graph) use unpublished_idempotency::WorthQueryUnpublishedIdempotencyReservation;

pub(super) use super::application_attempt::WorthQueryPrimaryGraphApplicationAttempt;
use super::WorthQueryPrimaryGraphIntegrationHandle;
pub(super) use application_attempt_state::WorthQueryPrimaryGraphCommittedApplication;
pub(crate) use application_attempt_work::WorthQueryApplicationAttemptWorkSnapshot;
pub(in crate::domain_computation) use application_decision_fact::WorthQueryPrimaryGraphApplicationDecisionFact;
#[cfg(test)]
pub(in crate::domain_computation) use committed_dispatch_outbox::{
    commit_distinct_records_and_admit_fixture, commit_observe_and_admit_fixture,
    commit_observe_and_admit_twice_fixture,
};
pub use committed_dispatch_outbox::{
    WorthQueryCommittedDispatchOutboxObservation, WorthQueryCommittedDispatchOutboxReadDenial,
    WorthQueryCommittedDispatchOutboxReadWork,
};
pub(in crate::domain_computation::primary_graph) use idempotency::WorthQueryProviderGuardedWorkflowOperationCustody;
pub(super) use idempotency::{
    WorthQueryProductIdempotencyAffinity, WorthQueryProviderIdempotencyResolution,
    WorthQueryProviderIdempotencyResolutionDenial,
};
pub use mutation_work::{WorthQueryPrimaryMutationWorkEvidence, WorthQueryTouchedRecordIdentity};
pub(in crate::domain_computation) use session_commit::WorthQueryCommittedDispatchOutboxBinding;
pub(crate) use session_commit::WorthQueryRetainedPreImageSeal;
pub(super) use session_commit::{
    WorthQueryCommittedDispatchOutboxBindingDenial, WorthQueryCommittedDispatchOutboxReceiptSeal,
    WorthQueryRetainedApplicationCommitBasis,
};
use std::sync::{Arc, Mutex};

pub(crate) struct WorthQueryPrimaryGraphProvider {
    pub(crate) graph: WorthQueryPrimaryGraphIntegrationHandle,
    resource_support: resource_support::WorthQueryPrimaryGraphResourceSupport,
    branch_commit_coordination:
        branch_commit_coordination::WorthQueryApplicationBranchCommitCoordinator,
    pub(super) live_delivery: super::live_delivery::WorthQueryLiveDeliverySource,
    attempts: Arc<Mutex<application_attempt_state::WorthQueryPrimaryGraphApplicationAttemptStore>>,
    #[cfg(feature = "test-world-operation-control")]
    pub(super) application_attempt_operation_control:
        super::application_runtime::WorthQueryApplicationAttemptOperationControl,
    application_attempt_work: application_attempt_work::WorthQueryApplicationAttemptWorkLedger,
    completed_commit_evidence: Mutex<session_commit::WorthQueryCompletedCommitEvidenceStore>,
    unpublished_idempotency:
        Arc<Mutex<unpublished_idempotency::WorthQueryUnpublishedIdempotencyStore>>,
    receipt_basis_retention: Arc<Mutex<session_commit::WorthQueryReceiptBasisRetentionStore>>,
    outstanding_dispatch: outstanding_dispatch::OutstandingDispatchOwner,
    inbound_terminal_index: inbound_terminal_index::WorthQueryInboundTerminalIndex,
    pending_application_publications:
        pending_application_publication::registry::WorthQueryPendingApplicationPublicationRegistryOwner,
    fault_port: Arc<dyn fault_port::WorthQueryPrimaryGraphFaultPort>,
    world_history: std::sync::OnceLock<worth_runtime_world::facade::RuntimeWorldLifecyclePort>,
}

pub(crate) use branch_commit_coordination::{
    WorthQueryApplicationBranchCommitCoordination, WorthQueryApplicationBranchCommitLane,
    WorthQueryBranchCommitLaneDenial,
};

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn reserve_outstanding_dispatch(
        &self,
        record: Option<
            &crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord,
        >,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Result<Option<OutstandingDispatchReservation>, &'static str> {
        let Some(record) = record else {
            return Ok(None);
        };
        self.outstanding_dispatch
            .reserve(record, observation)
            .map_err(|_| {
                "inbound dispatch provenance capacity or identity denied before owner effects"
            })
    }

    #[cfg(feature = "test-world-operation-control")]
    pub(super) fn after_application_attempt_registration_for_test(&self) {
        self.application_attempt_operation_control
            .after_registration();
    }

    #[cfg(feature = "test-world-operation-control")]
    pub(super) fn after_application_candidate_preparation_for_test(&self) {
        self.application_attempt_operation_control
            .after_candidate_preparation();
    }

    pub(super) fn application_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot
    {
        self.resource_support.snapshot()
    }

    pub(super) fn application_resource_support_ref(
        &self,
    ) -> &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot
    {
        self.resource_support.snapshot_ref()
    }

    pub(in crate::domain_computation::primary_graph) fn bind_application_idempotency_intent(
        &self,
        batch: worth_relational::facade::transactions::WorkerIntentBatch,
        idempotency: super::application_attempt::WorthQueryApplicationIdempotencyBinding,
        outcome_identity: super::application_attempt::WorthQueryApplicationCommitOutcomeIdentity,
        emitted_effect_count: u64,
        mutation_partition: worth_relational::facade::identity::PartitionId,
    ) -> worth_relational::facade::transactions::WorkerIntentBatch {
        batch.push(idempotency::idempotency_create_intent(
            self.graph.layout.provider_idempotency(),
            idempotency,
            outcome_identity,
            emitted_effect_count,
            mutation_partition,
        ))
    }

    pub(in crate::domain_computation::primary_graph) fn bind_application_aftermath_causality_intent(
        &self,
        batch: worth_relational::facade::transactions::WorkerIntentBatch,
        causality: &crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
        outcome_identity: super::application_attempt::WorthQueryApplicationCommitOutcomeIdentity,
        mutation_partition: worth_relational::facade::identity::PartitionId,
    ) -> worth_relational::facade::transactions::WorkerIntentBatch {
        batch.push(aftermath_causality::aftermath_causality_create_intent(
            self.graph.layout.provider_aftermath_causality(),
            causality,
            outcome_identity,
            mutation_partition,
        ))
    }

    pub(in crate::domain_computation::primary_graph) fn bind_application_dispatch_outbox(
        &self,
        mut batch: worth_relational::facade::transactions::WorkerIntentBatch,
        basis: dispatch_outbox::WorthQueryDispatchOutboxBasis<'_>,
        mutation_partition: worth_relational::facade::identity::PartitionId,
    ) -> Result<
        (
            worth_relational::facade::transactions::WorkerIntentBatch,
            Option<
                crate::domain_computation::application_aftermath::WorthQueryPendingDispatchOutbox,
            >,
        ),
        &'static str,
    > {
        let record = dispatch_outbox::derive_dispatch_outbox_record(basis)
            .map_err(|_| "external-effect correlation derivation failed")?;
        let pending =
            crate::domain_computation::application_aftermath::bind_dispatch_outbox_create_intent(
                Some(self.graph.layout.provider_dispatch_outbox()),
                record.as_ref(),
                mutation_partition,
            );
        if let Some((intent, _)) = &pending {
            batch = batch.push(intent.clone());
        }
        Ok((batch, pending.map(|(_, pending)| pending)))
    }

    pub(in crate::domain_computation) fn application_branch_commit_lane(
        &self,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Result<Arc<WorthQueryApplicationBranchCommitLane>, WorthQueryBranchCommitLaneDenial> {
        self.branch_commit_coordination.lane_for(observation)
    }

    pub(in crate::domain_computation::primary_graph) fn admitted_application_branch_commit_lane(
        &self,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        admission: &mut super::output_lineage::invalidation::InvalidationEditAdmission,
    ) -> Result<Arc<WorthQueryApplicationBranchCommitLane>, WorthQueryBranchCommitLaneDenial> {
        self.branch_commit_coordination
            .lane_for_admitted(observation, admission)
    }

    pub(crate) fn application_branch_commit_lane_for_occurrence(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> Result<Arc<WorthQueryApplicationBranchCommitLane>, WorthQueryBranchCommitLaneDenial> {
        self.branch_commit_coordination
            .lane_for_occurrence(occurrence)
    }

    pub(in crate::domain_computation::primary_graph) fn retire_application_branch_commit_lane(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) {
        self.branch_commit_coordination.retire(occurrence);
    }

    #[cfg(test)]
    pub(super) fn application_attempt_resource_count(&self) -> usize {
        self.attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .resource_count()
    }

    #[cfg(feature = "test-world-operation-control")]
    pub(super) fn active_application_attempt_count_for_test(&self) -> usize {
        self.attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .active_attempt_count()
    }

    #[cfg(feature = "test-world-operation-control")]
    pub(super) fn active_live_consumer_count_for_test(&self) -> usize {
        self.live_delivery.active_subscriber_count()
    }

    pub(super) fn application_attempt_work(&self) -> WorthQueryApplicationAttemptWorkSnapshot {
        self.application_attempt_work.snapshot()
    }

    pub(in crate::domain_computation::primary_graph) fn observe_managed_application_bridge_plan(
        &self,
    ) {
        self.application_attempt_work.observe_managed_bridge_plan();
    }

    pub(in crate::domain_computation::primary_graph) fn observe_managed_application_cleanup(&self) {
        self.application_attempt_work.observe_managed_cleanup();
    }

    pub(in crate::domain_computation::primary_graph) fn observe_external_dispatch_admission(&self) {
        self.application_attempt_work
            .observe_external_dispatch_admission();
    }
}

pub(super) struct WorthQueryPrimaryLogicalGraph;

#[cfg(test)]
pub(in crate::domain_computation::primary_graph) use inbound_completion::{
    completion_preparation_denial, completion_validation_denial,
};

pub(in crate::domain_computation::primary_graph) use application_attempt_state::{
    PreparedRebaseDenial, PreparedSourceFactRebase,
};
