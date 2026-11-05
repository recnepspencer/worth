//! Exact pending publication custody survives every retry and terminal cutover.
use super::{WorthQueryPrimaryGraphApplicationAttempt, WorthQueryPrimaryGraphCommittedApplication};
use crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationOwner;
use std::sync::Arc;
pub(in crate::domain_computation::primary_graph) struct WorthQueryPendingApplicationPublication {
    pub(super) predicate: Option<PredicatePreparationOwner>,
    pub(super) product_incarnation: worth_runtime_world::facade::ProductBranchIncarnation,
    pub(super) idempotency: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding,
    pub(super) attempt: Option<WorthQueryPrimaryGraphApplicationAttempt>,
    pub(super) before: Option<worth_relational::facade::snapshots::SnapshotHandle>,
    pub(super) next_basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    pub(super) committed: Arc<worth_relational::facade::transactions::CommitResult>,
    pub(super) application: Option<WorthQueryPrimaryGraphCommittedApplication>,
    pub(super) emitted_effect_count: usize,
    pub(super) outcome_identity: crate::domain_computation::primary_graph::vocabulary::application_attempt::commit_outcome_identity::WorthQueryApplicationCommitOutcomeIdentity,
    pub(super) aggregate_published: bool,
    pub(super) source_read_stop: Option<worth_relational::facade::transactions::RelationalExecutionDenialCause>,
    pub(super) receipt_basis_lease:
        Option<worth_relational::facade::branch::RelationalBranchRetentionLease>,
    pub(super) required_prerequisites:
        Option<crate::domain_computation::primary_graph::PreparedPrerequisiteClaims>,
    pub(super) prepared_lineage_slot:
        Option<crate::domain_computation::primary_graph::output_lineage::PreparedOutputLineageSlot>,

}

impl WorthQueryPendingApplicationPublication {
    pub(in crate::domain_computation::primary_graph) fn new(
        attempt: WorthQueryPrimaryGraphApplicationAttempt,
        before: worth_relational::facade::snapshots::SnapshotHandle,
        next_basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        committed: Arc<worth_relational::facade::transactions::CommitResult>,
        application: WorthQueryPrimaryGraphCommittedApplication,
        required_prerequisites: Option<
            crate::domain_computation::primary_graph::PreparedPrerequisiteClaims,
        >,
        prepared_lineage_slot: Option<
            crate::domain_computation::primary_graph::output_lineage::PreparedOutputLineageSlot,
        >,
    ) -> Self {
        let product_incarnation = application
            .committed_product_publication()
            .product_incarnation();
        let idempotency = application.commit_evidence().idempotency();
        let source_read_stop = application.commit_evidence().source_request_stop();
        let emitted_effect_count = attempt.emitted_effect_count();
        let outcome_identity = attempt.outcome_identity();
        Self {
            predicate: None,
            product_incarnation,
            idempotency,
            attempt: Some(attempt),
            before: Some(before),
            next_basis,
            committed,
            application: Some(application),
            emitted_effect_count,
            outcome_identity,
            aggregate_published: false,
            source_read_stop,
            receipt_basis_lease: None,
            required_prerequisites,
            prepared_lineage_slot,
        }
    }

    pub(super) fn release_before(
        &mut self,
        runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    ) {
        if let Some(before) = self.before.take() {
            crate::relational_snapshot_release::release_query_snapshot(runtime, &before);
        }
    }

    pub(super) const fn product_incarnation(
        &self,
    ) -> worth_runtime_world::facade::ProductBranchIncarnation {
        self.product_incarnation
    }

    pub(super) const fn idempotency(
        &self,
    ) -> crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding{
        self.idempotency
    }
}
