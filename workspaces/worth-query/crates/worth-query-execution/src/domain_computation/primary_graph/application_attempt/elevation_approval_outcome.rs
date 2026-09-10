use worth_relational::facade::identity::EntityId;

use super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationStaleAttempt,
    WorthQueryRequestedElevation,
};
use crate::domain_computation::authorization::{
    WorthQueryAuthorizationDecisionFact, WorthQueryElevationApprovalBinding,
    WorthQueryElevationRequestBinding, WorthQueryRetainedCapabilityRequest,
};

/// Move-only authority proving that Query committed the exact approval.
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryApprovedElevation;
///
/// fn approved_elevation_cannot_be_copied(receipt: WorthQueryApprovedElevation) {
///     let _copied = receipt.clone();
/// }
/// ```
#[derive(Debug)]
pub struct WorthQueryApprovedElevation {
    binding: WorthQueryElevationApprovalBinding,
    approval_commit: WorthQueryApplicationCommitReceipt,
}

impl WorthQueryApprovedElevation {
    pub(in crate::domain_computation) const fn approval_commit_receipt(
        &self,
    ) -> &WorthQueryApplicationCommitReceipt {
        &self.approval_commit
    }

    pub fn publication_source(&self) -> super::WorthQueryApplicationCommitPublicationSource {
        self.approval_commit.publication_source()
    }

    pub const fn approval_changed_record_count(&self) -> usize {
        self.approval_commit.changed_record_count()
    }

    pub const fn approval_emitted_effect_count(&self) -> usize {
        self.approval_commit.emitted_effect_count()
    }

    pub const fn approval_product_publication(
        &self,
    ) -> &super::WorthQueryCommittedProductPublication {
        self.approval_commit.committed_product_publication()
    }

    pub fn approval_retained_preimage(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage> {
        self.approval_commit.retained_preimage()
    }

    pub fn request_retained_preimage(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage> {
        self.binding.request_commit().retained_preimage()
    }

    pub fn approval_mutation_work(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryPrimaryMutationWorkEvidence>
    {
        self.approval_commit.mutation_work()
    }

    pub fn request_mutation_work(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryPrimaryMutationWorkEvidence>
    {
        self.binding.request_commit().mutation_work()
    }

    pub fn commits_share_branch(&self) -> bool {
        self.binding.request_commit().terminal().branch()
            == self.approval_commit.terminal().branch()
    }

    pub const fn requester(&self) -> EntityId {
        self.binding.requested().requester()
    }

    pub const fn approver(&self) -> EntityId {
        self.binding.approver()
    }

    pub const fn resource(&self) -> EntityId {
        self.binding.requested().resource()
    }

    pub const fn grant(&self) -> EntityId {
        self.binding.requested().grant()
    }

    pub const fn elevation(&self) -> EntityId {
        self.binding.elevation()
    }

    pub const fn review(&self) -> EntityId {
        self.binding.review()
    }

    pub const fn action(&self) -> &worth_foundational::facade::AspectValue {
        self.binding.requested().upper_bound().action()
    }

    pub const fn purpose(&self) -> &worth_foundational::facade::AspectValue {
        self.binding.requested().upper_bound().purpose()
    }

    pub const fn field(&self) -> Option<&worth_foundational::facade::AspectValue> {
        self.binding.requested().upper_bound().field()
    }

    pub const fn magnitude(&self) -> Option<&worth_foundational::facade::AspectValue> {
        self.binding.requested().upper_bound().magnitude()
    }

    pub const fn cardinality(&self) -> u32 {
        self.binding.requested().upper_bound().cardinality()
    }

    pub const fn reason(&self) -> &worth_foundational::facade::AspectValue {
        self.binding.requested().reason()
    }

    pub const fn issued_at(&self) -> &worth_foundational::facade::AspectValue {
        self.binding.requested().issued_at()
    }

    pub const fn expires_at(&self) -> &worth_foundational::facade::AspectValue {
        self.binding.requested().expires_at()
    }

    pub(in crate::domain_computation) fn belongs_to_lifecycle(
        &self,
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        branch: &worth_relational::facade::history::BranchId,
        capability_identity: [u8; 32],
        capability_authority_identity: &str,
    ) -> bool {
        *self.binding.requested().runtime_authority() == runtime_authority
            && self.binding.requested().branch() == branch
            && self.binding.requested().capability_identity() == capability_identity
            && self.binding.requested().capability_authority_identity()
                == capability_authority_identity
            && self.binding.request_commit().terminal().branch() == branch
            && self.approval_commit.terminal().branch() == branch
            && self.binding.request_commit().provider_runtime_instance_id()
                == self.approval_commit.provider_runtime_instance_id()
    }

    pub(in crate::domain_computation) const fn request_binding(
        &self,
    ) -> &WorthQueryElevationRequestBinding {
        self.binding.requested()
    }

    pub(in crate::domain_computation) fn support_remains_current_in(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        bridge: &worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
    ) -> bool {
        self.binding
            .requested()
            .supporting()
            .decision()
            .remains_current_in(runtime, snapshot, bridge)
    }

    pub(in crate::domain_computation) fn support_decision(
        &self,
    ) -> &WorthQueryAuthorizationDecisionFact {
        self.binding.requested().supporting().decision()
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation) fn admits_active_use(
        &self,
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        branch: &worth_relational::facade::history::BranchId,
        capability_identity: [u8; 32],
        capability_authority_identity: &str,
        request: &WorthQueryRetainedCapabilityRequest,
        elevation: EntityId,
        grant: EntityId,
    ) -> bool {
        self.belongs_to_lifecycle(
            runtime_authority,
            branch,
            capability_identity,
            capability_authority_identity,
        ) && self.binding.requested().upper_bound().capability_identity() == capability_identity
            && self.binding.elevation() == elevation
            && self
                .binding
                .requested()
                .upper_bound()
                .matches_active_request(request, elevation, grant)
    }
}

#[derive(Debug)]
pub enum WorthQueryElevationApprovalOutcome {
    ProductStale(
        crate::domain_computation::WorthQueryProductStaleApplication,
        WorthQueryRequestedElevation,
    ),
    ProductUnpublished(crate::domain_computation::WorthQueryProductUnpublishedApplication),
    NoEffect(
        super::WorthQueryApplicationNoEffect,
        WorthQueryRequestedElevation,
    ),
    Approved(WorthQueryApprovedElevation),
    AlreadyApproved(WorthQueryApprovedElevation),
    Stale(
        WorthQueryApplicationStaleAttempt,
        WorthQueryRequestedElevation,
    ),
    Cancelled(WorthQueryRequestedElevation),
    TimedOut,
    Denied(
        WorthQueryApplicationCommitDenial,
        WorthQueryRequestedElevation,
    ),
    Aborted(WorthQueryRequestedElevation),
    Deferred(super::WorthQueryApplicationCommitDeferred),
    SettlementDeferred(super::WorthQueryApplicationSettlementDeferred),
    Indeterminate,
}

pub(in crate::domain_computation::primary_graph) fn approved_outcome(
    outcome: WorthQueryApplicationCommitOutcome,
    binding: WorthQueryElevationApprovalBinding,
) -> WorthQueryElevationApprovalOutcome {
    match outcome {
        WorthQueryApplicationCommitOutcome::Committed(commit) => {
            WorthQueryElevationApprovalOutcome::Approved(approved(binding, commit))
        }
        WorthQueryApplicationCommitOutcome::AlreadyCommitted(commit) => {
            WorthQueryElevationApprovalOutcome::AlreadyApproved(approved(binding, commit))
        }
        WorthQueryApplicationCommitOutcome::Stale(stale) => {
            WorthQueryElevationApprovalOutcome::Stale(stale, binding.into_requested())
        }
        WorthQueryApplicationCommitOutcome::ProductStale(stale) => {
            WorthQueryElevationApprovalOutcome::ProductStale(stale, binding.into_requested())
        }
        WorthQueryApplicationCommitOutcome::Cancelled => {
            WorthQueryElevationApprovalOutcome::Cancelled(binding.into_requested())
        }
        WorthQueryApplicationCommitOutcome::TimedOut => {
            WorthQueryElevationApprovalOutcome::TimedOut
        }
        WorthQueryApplicationCommitOutcome::Denied(denial) => {
            WorthQueryElevationApprovalOutcome::Denied(denial, binding.into_requested())
        }
        WorthQueryApplicationCommitOutcome::Aborted => {
            WorthQueryElevationApprovalOutcome::Aborted(binding.into_requested())
        }
        WorthQueryApplicationCommitOutcome::Deferred(deferred) => {
            WorthQueryElevationApprovalOutcome::Deferred(deferred)
        }
        WorthQueryApplicationCommitOutcome::ProductUnpublished(unpublished) => {
            WorthQueryElevationApprovalOutcome::ProductUnpublished(unpublished)
        }
        WorthQueryApplicationCommitOutcome::NoEffect(no_effect) => {
            WorthQueryElevationApprovalOutcome::NoEffect(no_effect, binding.into_requested())
        }
        WorthQueryApplicationCommitOutcome::SettlementDeferred(deferred) => {
            WorthQueryElevationApprovalOutcome::SettlementDeferred(deferred)
        }
        WorthQueryApplicationCommitOutcome::Indeterminate(_) => {
            WorthQueryElevationApprovalOutcome::Indeterminate
        }
    }
}

fn approved(
    binding: WorthQueryElevationApprovalBinding,
    approval_commit: WorthQueryApplicationCommitReceipt,
) -> WorthQueryApprovedElevation {
    WorthQueryApprovedElevation {
        binding,
        approval_commit,
    }
}
