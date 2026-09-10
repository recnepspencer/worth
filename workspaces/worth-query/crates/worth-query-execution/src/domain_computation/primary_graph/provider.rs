mod aftermath_causality;
pub(in crate::domain_computation) use aftermath_causality::WorthQueryAftermathCausalityReadDenial;
mod application_attempt_state;
mod application_attempt_work;
mod application_decision_fact;
mod application_touch_admission;
mod branch_commit_coordination;
mod commit_causality;
pub(super) mod committed_dispatch_outbox;
mod conditional_commit_journal;
mod decision_facts;
pub(in crate::domain_computation::primary_graph) mod dispatch_outbox;
pub(in crate::domain_computation::primary_graph) mod fault_port;
mod graph_participation;
mod idempotency;
mod installation;
mod invariant_execution;
mod invariant_execution_failure;
mod mutation_work;
mod pending_application_publication;
pub(in crate::domain_computation::primary_graph) use pending_application_publication::WorthQueryApplicationPublicationRecoveryReservation;
mod product_retirement;
mod provisional_state;
mod publication_recovery;
mod resource_support;
mod session_commit;
mod session_lifecycle;
mod unpublished_idempotency;
#[cfg(all(test, feature = "test-world-operation-control"))]
pub(in crate::domain_computation::primary_graph) use unpublished_idempotency::unwind_recovery_inspection_count;
pub(in crate::domain_computation) use unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition;

use std::sync::{Arc, Mutex};

pub(super) use super::application_attempt::WorthQueryPrimaryGraphApplicationAttempt;
use super::WorthQueryPrimaryGraphIntegrationHandle;
pub(super) use application_attempt_state::WorthQueryPrimaryGraphCommittedApplication;
#[cfg(test)]
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
    receipt_basis_retention: Mutex<session_commit::WorthQueryReceiptBasisRetentionStore>,
    pending_application_publications:
        pending_application_publication::registry::WorthQueryPendingApplicationPublicationRegistryOwner,
    conditional_commit_journal:
        Mutex<conditional_commit_journal::WorthQueryConditionalCommitJournal>,
    fault_port: Arc<dyn fault_port::WorthQueryPrimaryGraphFaultPort>,
}

pub(in crate::domain_computation) use branch_commit_coordination::{
    WorthQueryApplicationBranchCommitCoordination, WorthQueryApplicationBranchCommitLane,
};

impl WorthQueryPrimaryGraphProvider {
    #[cfg(feature = "test-world-operation-control")]
    pub(super) fn after_application_attempt_registration_for_test(&self) {
        self.application_attempt_operation_control
            .after_registration();
    }

    pub(in crate::domain_computation::primary_graph) fn conditional_commit_sequence(&self) -> u64 {
        self.conditional_commit_journal
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .latest_sequence()
    }

    pub(in crate::domain_computation::primary_graph) fn replace_conditional_commit_routes(
        &self,
        records: impl IntoIterator<Item = worth_relational::facade::transactions::RecordRef>,
        include_whole_graph: bool,
        bootstrap_identities: impl IntoIterator<Item = String>,
    ) {
        let mut journal = self
            .conditional_commit_journal
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        journal.replace_routes(records, include_whole_graph);
        journal.replace_bootstrap_routes(bootstrap_identities);
    }

    pub(in crate::domain_computation::primary_graph) fn record_conditional_commit(
        &self,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
        records: impl IntoIterator<Item = worth_relational::facade::transactions::RecordRef>,
    ) {
        self.conditional_commit_journal
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .record(commit, records);
    }

    pub(in crate::domain_computation::primary_graph) fn conditional_commits_after_records(
        &self,
        branch: &worth_relational::facade::history::BranchId,
        commit_ceiling: Option<worth_relational::facade::history::CommitId>,
        sequence: u64,
        maximum: usize,
        records: impl IntoIterator<Item = worth_relational::facade::transactions::RecordRef>,
        include_whole_graph: bool,
        bootstrap_identity: Option<&str>,
    ) -> Result<conditional_commit_journal::WorthQueryConditionalCommitBatch, &'static str> {
        self.conditional_commit_journal
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .after_records_with_bootstrap(
                branch,
                commit_ceiling,
                sequence,
                maximum,
                records,
                include_whole_graph,
                bootstrap_identity,
            )
    }

    pub(in crate::domain_computation::primary_graph) fn narrow_conditional_bootstrap_route_if_current(
        &self,
        identity: &str,
        expected_frontier: u64,
    ) -> bool {
        self.conditional_commit_journal
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .narrow_bootstrap_route_if_current(identity, expected_frontier)
    }

    pub(super) fn application_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot
    {
        self.resource_support.snapshot()
    }

    pub(in crate::domain_computation::primary_graph) fn bind_application_idempotency_intent(
        &self,
        batch: worth_relational::facade::transactions::WorkerIntentBatch,
        idempotency: super::application_attempt::WorthQueryApplicationIdempotencyBinding,
        outcome_identity: super::application_attempt::WorthQueryApplicationCommitOutcomeIdentity,
        emitted_effect_count: u64,
    ) -> worth_relational::facade::transactions::WorkerIntentBatch {
        batch.push(idempotency::idempotency_create_intent(
            self.graph.layout.provider_idempotency(),
            idempotency,
            outcome_identity,
            emitted_effect_count,
        ))
    }

    pub(in crate::domain_computation::primary_graph) fn bind_application_aftermath_causality_intent(
        &self,
        batch: worth_relational::facade::transactions::WorkerIntentBatch,
        causality: &crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
        outcome_identity: super::application_attempt::WorthQueryApplicationCommitOutcomeIdentity,
    ) -> worth_relational::facade::transactions::WorkerIntentBatch {
        batch.push(aftermath_causality::aftermath_causality_create_intent(
            self.graph.layout.provider_aftermath_causality(),
            causality,
            outcome_identity,
        ))
    }

    pub(in crate::domain_computation::primary_graph) fn bind_application_dispatch_outbox(
        &self,
        mut batch: worth_relational::facade::transactions::WorkerIntentBatch,
        basis: dispatch_outbox::WorthQueryDispatchOutboxBasis<'_>,
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
            );
        if let Some((intent, _)) = &pending {
            batch = batch.push(intent.clone());
        }
        Ok((batch, pending.map(|(_, pending)| pending)))
    }

    pub(in crate::domain_computation) fn application_branch_commit_lane(
        &self,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Arc<WorthQueryApplicationBranchCommitLane> {
        self.branch_commit_coordination.lane_for(observation)
    }

    pub(in crate::domain_computation::primary_graph) fn application_branch_commit_lane_for_occurrence(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> Arc<WorthQueryApplicationBranchCommitLane> {
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

    #[cfg(test)]
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

    pub(super) fn take_lost_commit_response(&self) -> bool {
        self.take_fault(fault_port::WorthQueryPrimaryGraphFault::LostCommitResponse)
    }

    pub(super) fn take_rejected_session_prepare(&self) -> bool {
        self.take_fault(fault_port::WorthQueryPrimaryGraphFault::RejectedSessionPreparation)
    }

    pub(super) fn take_rejected_commit_before_transaction(&self) -> bool {
        self.take_fault(fault_port::WorthQueryPrimaryGraphFault::RejectedCommitBeforeTransaction)
    }

    pub(super) fn take_failed_index_publication(&self) -> bool {
        self.take_fault(fault_port::WorthQueryPrimaryGraphFault::FailedIndexPublication)
    }

    pub(super) fn take_failed_post_commit_snapshot(&self) -> bool {
        self.take_fault(fault_port::WorthQueryPrimaryGraphFault::FailedPostCommitSnapshot)
    }

    #[cfg(feature = "test-primary-graph-faults")]
    pub(super) fn fail_next_index_publication_for_test(&self) {
        assert!(self
            .fault_port
            .schedule_for_test(fault_port::WorthQueryPrimaryGraphFault::FailedIndexPublication));
    }

    pub(super) fn observe_completed_application(
        &self,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Option<WorthQueryPrimaryGraphCommittedApplication> {
        self.completed_commit_evidence
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .observe(commit)
    }

    pub(in crate::domain_computation::primary_graph) fn observe_completed_application_for_session(
        &self,
        session: &crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    ) -> Option<WorthQueryPrimaryGraphCommittedApplication> {
        self.completed_commit_evidence
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .observe_session(session)
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn retained_application_commit_basis(
        &self,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Option<WorthQueryRetainedApplicationCommitBasis> {
        self.completed_commit_evidence
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .observe(commit)?;
        self.receipt_basis_retention
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .acquire(commit.commit_id)
    }

    pub(in crate::domain_computation::primary_graph) fn resolve_completed_application_idempotency(
        &self,
        product: &WorthQueryProductIdempotencyAffinity,
        binding: super::application_attempt::WorthQueryApplicationIdempotencyBinding,
    ) -> Option<WorthQueryProviderIdempotencyResolution> {
        let (committed_binding, committed) = self
            .completed_commit_evidence
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .observe_idempotency(product, binding)?;
        Some(if committed_binding == binding {
            WorthQueryProviderIdempotencyResolution::Equivalent(committed)
        } else {
            WorthQueryProviderIdempotencyResolution::Drift
        })
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn retained_receipt_basis_count(
        &self,
    ) -> usize {
        self.receipt_basis_retention
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .retained_count()
    }

    pub(super) fn take_skipped_invariant_owner_execution(&self) -> bool {
        self.take_fault(fault_port::WorthQueryPrimaryGraphFault::SkippedInvariantOwnerExecution)
    }

    pub(super) fn take_relational_invariant_violation(&self) -> bool {
        self.take_fault(fault_port::WorthQueryPrimaryGraphFault::RelationalInvariantViolation)
    }

    #[cfg(test)]
    pub(super) fn take_undeclared_application_touch(&self) -> bool {
        self.take_fault(fault_port::WorthQueryPrimaryGraphFault::UndeclaredApplicationTouch)
    }

    #[cfg(test)]
    pub(super) fn take_panicked_pending_application_publication(&self) -> bool {
        self.take_fault(
            fault_port::WorthQueryPrimaryGraphFault::PanickedPendingApplicationPublication,
        )
    }

    fn take_fault(&self, fault: fault_port::WorthQueryPrimaryGraphFault) -> bool {
        self.fault_port.take(fault)
    }
}

pub(super) struct WorthQueryPrimaryLogicalGraph;
