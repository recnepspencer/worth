use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
pub use worth_query_execution_lineage::facade::primary_graph::provider::{
    branch_commit_coordination, invariant_execution_failure, native_publication_roles,
    relational_execution_denial,
};
pub mod aftermath_causality;
pub mod authored_retirement_selection;
pub mod mutation_work;
pub mod original_attempt_completion;
pub mod publication_posture;
pub use aftermath_causality::WorthQueryAftermathCausalityReadDenial;
pub use aftermath_causality::{
    WorthQueryActiveReversalProgress, WorthQueryCanonicalCorrectionFact,
    WorthQueryCorrectionHistoryPageAllocation, WorthQueryCorrectionHistoryProgress,
};
pub mod application_attempt_state;
pub use application_attempt_state::publish_recovered;
pub use application_attempt_state::ManagedUnpublishedAttempt;
pub use application_attempt_state::RebaseVerificationFailure;
pub use application_attempt_state::RegisteredPrimaryRead;
pub use application_attempt_state::commit_preparation::relational_commit::commit_execution::executed_adoption::PreparedExecutedOutputAdoption;
pub use application_attempt_state::{
    AppliedOutputParticipants, AppliedParticipantOutputs, CandidateReadSupport,
    CoupledApplicationMember, CoupledApplicationMemberKind, CoupledAttemptPreparation,
    CoupledProducerPreparation, ParentRead,
};
pub use application_attempt_state::{
    FactlessCurrentness, OwnEffectOnReads, RebaseVerificationReason,
};
pub mod application_attempt_work;
pub mod application_decision_fact;
pub mod application_touch_admission;
pub mod commit_causality;
pub mod committed_dispatch_outbox;
pub mod completed_observation;
pub use original_attempt_completion::{
    OriginalCompletionPublisher, OriginalCompletionReceiver, OriginalCompletionSlot,
};
pub mod correction_witness;
pub use correction_witness::{CorrectionPerformedWitness, CorrectionWitnessDenial};
pub mod correction_idempotency;
pub use correction_idempotency::CorrectionIdempotencyObservation;
pub mod decision_facts;
pub use decision_facts::PrimaryDecisionReadSetComparison;
pub mod dispatch_outbox;
pub mod evidence_window;
pub mod fault_port;
pub mod graph_participation;
pub mod home_retirement;
pub use home_retirement::PrimaryProviderCustody;
pub mod correction_material;
pub mod idempotency;
pub use idempotency::ReceiptAppendix;
pub mod original_meaning_support;
pub mod original_publication;
pub use idempotency::{
    WorthQueryRecoveredAftermathCausality, WorthQueryRecoveredApplicationCommitReceipt,
    WorthQueryRecoveredDispatchOutbox, WorthQueryReleasedApplicationEffect,
};
use original_publication::OriginalPublication;
pub mod inbound_completion;
pub mod inbound_cost;
pub mod managed_cleanup_control;
pub mod publication_fault_control;
pub use inbound_completion::WorthQueryCanonicalCompletionRow;
pub use inbound_completion::WorthQueryInboundCompletionPreparationDenial;
pub use inbound_completion::WorthQueryInboundCompletionReadDenial;
pub mod inbound_terminal_index;
pub use inbound_terminal_index::WorthQueryCanonicalInboundCompletion;
pub use inbound_terminal_index::WorthQueryInboundTerminalIndexDenial;
pub mod coupled_members;
pub mod installation;
pub mod invariant_execution;
pub use invariant_execution::ClosedCoupledInvariantReceipts;
pub use invariant_execution::CoupledProducerAssembly;
pub mod inspection_fault;
pub mod output_readiness_fault;
pub mod outstanding_dispatch;
pub use outstanding_dispatch::OutstandingDispatchInFlightLease;
pub use outstanding_dispatch::WorthQueryOutstandingInFlightDenial;
pub use outstanding_dispatch::WorthQueryTerminalDispatchReleaseDenial;
pub mod pending_application_publication;
pub use outstanding_dispatch::OutstandingDispatchReservation;
pub use pending_application_publication::WorthQueryApplicationPublicationRecoveryReservation;
pub mod product_retirement;
pub mod provisional_state;
pub mod resource_support;
pub use resource_support::WorthQueryPrimaryGraphResourceSupport;
pub mod session_commit;
pub mod session_lifecycle;
pub mod terminal_dispatch_release;
pub mod unpublished_idempotency;
pub mod unpublished_idempotency_port;

pub use unpublished_idempotency::QueryDurablePreparationGuard;
pub use unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition;
pub use unpublished_idempotency::WorthQueryUnpublishedIdempotencyReservation;

pub use super::application_attempt::WorthQueryPrimaryGraphApplicationAttempt;
use super::WorthQueryPrimaryGraphIntegrationHandle;
pub use application_attempt_state::WorthQueryPrimaryGraphCommittedApplication;
pub use application_attempt_work::WorthQueryApplicationAttemptWorkSnapshot;
pub use application_decision_fact::ApplicationFactTable;
pub use application_decision_fact::CompletedApplicationFacts;
pub use application_decision_fact::WorthQueryPrimaryGraphApplicationDecisionFact;

pub use committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxObservation;
pub use committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxReadDenial;
pub use committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxReadWork;
pub use idempotency::WorthQueryProductIdempotencyAffinity;
pub use idempotency::WorthQueryProviderGuardedWorkflowOperationCustody;
pub use idempotency::WorthQueryProviderIdempotencyResolution;
pub use idempotency::WorthQueryProviderIdempotencyResolutionDenial;
pub use mutation_work::WorthQueryPrimaryMutationWorkEvidence;
pub use mutation_work::WorthQueryTouchedRecordIdentity;
pub use session_commit::WorthQueryCommittedDispatchOutboxBinding;
pub use session_commit::WorthQueryCommittedDispatchOutboxBindingDenial;
pub use session_commit::WorthQueryCommittedDispatchOutboxReceiptSeal;
pub use session_commit::{WorthQueryRetainedPreImageSeal, WorthQueryRetainedRelationPreImageSeal};
use std::sync::Arc;
use std::sync::Mutex;

pub struct WorthQueryPrimaryGraphProvider {
    pub managed_cleanup_control: Arc<managed_cleanup_control::ManagedCleanupControl>,
    pub graph: WorthQueryPrimaryGraphIntegrationHandle,
    pub installed_operations: Arc<worth_query_installation::facade::InstalledOperationCatalog>,
    pub resource_support: resource_support::WorthQueryPrimaryGraphResourceSupport,
    pub branch_commit_coordination:
        branch_commit_coordination::WorthQueryApplicationBranchCommitCoordinator,
    pub live_delivery: super::live_delivery::WorthQueryLiveDeliverySource,
    pub attempts: Arc<Mutex<application_attempt_state::WorthQueryPrimaryGraphApplicationAttemptStore>>,
    pub application_attempt_operation_control:
        super::application_runtime::WorthQueryApplicationAttemptOperationControl,
    pub application_attempt_work: application_attempt_work::WorthQueryApplicationAttemptWorkLedger,
    pub completed_commit_evidence: Mutex<session_commit::WorthQueryCompletedCommitEvidenceStore>,
    pub unpublished_idempotency:
        Arc<Mutex<unpublished_idempotency::WorthQueryUnpublishedIdempotencyStore>>,
    pub receipt_basis_retention: Arc<Mutex<session_commit::WorthQueryReceiptBasisRetentionStore>>,
    pub outstanding_dispatch: outstanding_dispatch::OutstandingDispatchOwner,
    pub inbound_terminal_index: inbound_terminal_index::WorthQueryInboundTerminalIndex,
    pub pending_application_publications:
        pending_application_publication::registry::WorthQueryPendingApplicationPublicationRegistryOwner,
    pub fault_port: Arc<dyn fault_port::WorthQueryPrimaryGraphFaultPort>,
    pub publication_fault_control: Arc<publication_fault_control::PublicationFaultControl>,
    pub correction_material: std::sync::OnceLock<super::retained_correction::CorrectionMaterialOwner>,
    pub original_meaning_support: std::sync::OnceLock<std::sync::Weak<super::query_obligations::history::QueryObligationHistory>>,
    pub world_history: std::sync::OnceLock<worth_runtime_world::facade::RuntimeWorldRecoveryPort>,
    pub managed_views_execution: std::sync::OnceLock<Arc<dyn crate::ports::ManagedDerivedViewExecutionPort>>,
    pub teardown_sentinel: Mutex<Option<Box<dyn std::any::Any + Send + Sync>>>,
}

pub use branch_commit_coordination::WorthQueryApplicationBranchCommitContentionForTest;

pub use branch_commit_coordination::WorthQueryApplicationBranchCommitCoordination;
pub use branch_commit_coordination::WorthQueryApplicationBranchCommitLane;

impl WorthQueryPrimaryGraphProvider {
    pub fn install_managed_views_execution(
        &self,
        port: Arc<dyn crate::ports::ManagedDerivedViewExecutionPort>,
    ) {
        let _ = self.managed_views_execution.set(port);
    }

    pub(super) fn managed_views_execution(
        &self,
    ) -> Option<&Arc<dyn crate::ports::ManagedDerivedViewExecutionPort>> {
        self.managed_views_execution.get()
    }

    pub fn reserve_outstanding_dispatch(
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

    pub fn after_application_attempt_registration_for_test(&self) {
        self.application_attempt_operation_control
            .after_registration();
    }

    pub(super) fn after_application_candidate_preparation_for_test(&self) {
        self.application_attempt_operation_control
            .after_candidate_preparation();
    }

    pub fn application_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot
    {
        self.resource_support.snapshot()
    }

    pub fn application_resource_support_ref(
        &self,
    ) -> &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot
{
        self.resource_support.snapshot_ref()
    }

    pub fn bind_application_idempotency_intent(
        &self,
        batch: worth_relational::facade::transactions::WorkerIntentBatch,
        idempotency: super::application_attempt::WorthQueryApplicationIdempotencyBinding,
        outcome_identity: super::application_attempt::WorthQueryApplicationCommitOutcomeIdentity,
        emitted_effect_count: u64,
        mutation_partition: worth_relational::facade::identity::PartitionId,
        product: &worth_runtime_world::facade::ProductBranchObservation,
        has_output_settlement: bool,
    ) -> (
        worth_relational::facade::transactions::WorkerIntentBatch,
        idempotency::ReceiptAppendix,
    ) {
        idempotency::bind_receipt(
            batch,
            self.graph.layout_ref().provider_idempotency(),
            idempotency,
            outcome_identity,
            emitted_effect_count,
            mutation_partition,
            product,
            has_output_settlement,
        )
    }

    pub fn bind_application_aftermath_causality_intent(
        &self,
        batch: worth_relational::facade::transactions::WorkerIntentBatch,
        causality: &crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
        outcome_identity: super::application_attempt::WorthQueryApplicationCommitOutcomeIdentity,
        mutation_partition: worth_relational::facade::identity::PartitionId,
    ) -> worth_relational::facade::transactions::WorkerIntentBatch {
        batch.push(aftermath_causality::aftermath_causality_create_intent(
            self.graph.layout_ref().provider_aftermath_causality(),
            causality,
            outcome_identity,
            mutation_partition,
        ))
    }

    pub fn bind_application_dispatch_outbox(
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
                Some(self.graph.layout_ref().provider_dispatch_outbox()),
                record.as_ref(),
                mutation_partition,
            );
        if let Some((intent, _)) = &pending {
            batch = batch.push(intent.clone());
        }
        Ok((batch, pending.map(|(_, pending)| pending)))
    }

    pub fn observe_branch_commit_contention_for_test(
        &self,
    ) -> WorthQueryApplicationBranchCommitContentionForTest {
        self.branch_commit_coordination
            .observe_contention_for_test()
    }

    pub fn application_branch_commit_lane(
        &self,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Arc<WorthQueryApplicationBranchCommitLane> {
        self.branch_commit_coordination.lane_for(observation)
    }

    pub fn application_branch_commit_lane_for_occurrence(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> Arc<WorthQueryApplicationBranchCommitLane> {
        self.branch_commit_coordination
            .lane_for_occurrence(occurrence)
    }

    pub fn retire_application_branch_commit_lane(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) {
        self.branch_commit_coordination.retire(occurrence);
    }

    pub fn application_attempt_resource_count(&self) -> usize {
        self.attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .resource_count()
    }

    pub fn active_application_attempt_count_for_test(&self) -> usize {
        self.attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .active_attempt_count()
    }

    pub fn active_live_consumer_count_for_test(&self) -> usize {
        self.live_delivery.active_subscriber_count()
    }

    pub fn application_attempt_work_for_test(&self) -> WorthQueryApplicationAttemptWorkSnapshot {
        self.application_attempt_work.snapshot()
    }

    pub fn observe_managed_application_bridge_plan(&self) {
        self.application_attempt_work.observe_managed_bridge_plan();
    }

    pub fn observe_managed_application_cleanup(&self) {
        self.application_attempt_work.observe_managed_cleanup();
    }

    pub fn observe_external_dispatch_admission(&self) {
        self.application_attempt_work
            .observe_external_dispatch_admission();
    }
}

pub struct WorthQueryPrimaryLogicalGraph;

pub use application_attempt_state::PreparedRebaseDenial;
pub use application_attempt_state::PreparedSourceFactRebase;

pub use managed_cleanup_control::WorthQueryManagedCleanupRegistrationDenial;
pub use managed_cleanup_control::WorthQueryManagedCleanupScheduleForTest;

pub use application_attempt_state::QueryPreparingApplicationCommit;
pub use application_attempt_state::RetainedQueryDurablePreparation;

pub use unpublished_idempotency::BeforeWorldQueryReservation;
pub mod court_doors_for_test;

pub mod correction_lanes;
pub mod history_retirement;

#[cfg(test)]
#[path = "provider_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
