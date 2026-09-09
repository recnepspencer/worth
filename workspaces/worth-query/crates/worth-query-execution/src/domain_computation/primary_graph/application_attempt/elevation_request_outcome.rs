use worth_foundational::facade::AspectValue;
use worth_relational::facade::identity::EntityId;

use super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationStaleAttempt,
};
use crate::domain_computation::authorization::{
    WorthQueryElevationApprovalBindingPermit, WorthQueryElevationRequestBinding,
};

/// Exact, move-only evidence that Query committed one requested elevation.
///
/// The receipt is descriptive until a later lifecycle transition consumes it;
/// it cannot itself authorize active elevated use.
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryRequestedElevation;
///
/// fn requested_elevation_cannot_be_copied(receipt: WorthQueryRequestedElevation) {
///     let _copied = receipt.clone();
/// }
/// ```
#[derive(Debug)]
pub struct WorthQueryRequestedElevation {
    binding: WorthQueryElevationRequestBinding,
    commit: WorthQueryApplicationCommitReceipt,
}

impl WorthQueryRequestedElevation {
    pub(in crate::domain_computation) const fn commit_receipt(
        &self,
    ) -> &WorthQueryApplicationCommitReceipt {
        &self.commit
    }

    pub fn publication_source(&self) -> super::WorthQueryApplicationCommitPublicationSource {
        self.commit.publication_source()
    }

    pub const fn capability_identity(&self) -> [u8; 32] {
        self.binding.capability_identity()
    }

    pub fn capability_authority_identity(&self) -> &str {
        self.binding.capability_authority_identity()
    }

    pub const fn requester(&self) -> EntityId {
        self.binding.requester()
    }

    pub const fn resource(&self) -> EntityId {
        self.binding.resource()
    }

    pub const fn grant(&self) -> EntityId {
        self.binding.grant()
    }

    pub const fn action(&self) -> &AspectValue {
        self.binding.upper_bound().action()
    }

    pub const fn purpose(&self) -> &AspectValue {
        self.binding.upper_bound().purpose()
    }

    pub const fn field(&self) -> Option<&AspectValue> {
        self.binding.upper_bound().field()
    }

    pub const fn magnitude(&self) -> Option<&AspectValue> {
        self.binding.upper_bound().magnitude()
    }

    pub const fn cardinality(&self) -> u32 {
        self.binding.upper_bound().cardinality()
    }

    pub fn elevation_key(&self) -> &str {
        self.binding.elevation_key()
    }

    pub const fn elevation_identity(&self) -> &AspectValue {
        self.binding.elevation_identity()
    }

    pub const fn reason(&self) -> &AspectValue {
        self.binding.reason()
    }

    pub const fn requested_status(&self) -> &AspectValue {
        self.binding.requested_status()
    }

    pub const fn issued_at(&self) -> &AspectValue {
        self.binding.issued_at()
    }

    pub const fn expires_at(&self) -> &AspectValue {
        self.binding.expires_at()
    }

    pub fn review_key(&self) -> &str {
        self.binding.review_key()
    }

    pub const fn review_identity(&self) -> &AspectValue {
        self.binding.review_identity()
    }

    pub const fn review_status(&self) -> &AspectValue {
        self.binding.review_required_status()
    }

    const fn new(
        binding: WorthQueryElevationRequestBinding,
        commit: WorthQueryApplicationCommitReceipt,
    ) -> Self {
        Self { binding, commit }
    }

    pub(in crate::domain_computation) const fn binding(
        &self,
    ) -> &WorthQueryElevationRequestBinding {
        &self.binding
    }

    pub(in crate::domain_computation) fn apply_current_support<
        Schema,
        Capability,
        Operation,
        Input,
    >(
        &mut self,
        current: crate::domain_computation::authorization::WorthQueryCurrentElevationSupport,
        access: &mut crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        subject: &str,
    ) -> Result<
        crate::domain_computation::authorization::WorthQueryRuntimeTimeSample,
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    >
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
        Input:
            worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest<
                Schema,
                Capability,
            >,
    {
        self.binding.apply_current_support(current, access, subject)
    }

    pub(in crate::domain_computation) fn into_approval_parts(
        self,
        _permit: WorthQueryElevationApprovalBindingPermit,
    ) -> (
        WorthQueryElevationRequestBinding,
        WorthQueryApplicationCommitReceipt,
    ) {
        (self.binding, self.commit)
    }

    pub(in crate::domain_computation) const fn restore_after_approval(
        binding: WorthQueryElevationRequestBinding,
        commit: WorthQueryApplicationCommitReceipt,
        _permit: WorthQueryElevationApprovalBindingPermit,
    ) -> Self {
        Self { binding, commit }
    }
}

#[derive(Debug)]
pub enum WorthQueryElevationRequestOutcome {
    ProductStale(crate::domain_computation::WorthQueryProductStaleApplication),
    ProductUnpublished(crate::domain_computation::WorthQueryProductUnpublishedApplication),
    Requested(WorthQueryRequestedElevation),
    AlreadyRequested(WorthQueryRequestedElevation),
    Stale(WorthQueryApplicationStaleAttempt),
    Cancelled,
    TimedOut,
    Denied(WorthQueryApplicationCommitDenial),
    Aborted,
    Deferred(super::WorthQueryApplicationCommitDeferred),
    SettlementDeferred(super::WorthQueryApplicationSettlementDeferred),
    Indeterminate,
}

pub(in crate::domain_computation::primary_graph) fn requested_outcome(
    outcome: WorthQueryApplicationCommitOutcome,
    binding: WorthQueryElevationRequestBinding,
) -> WorthQueryElevationRequestOutcome {
    match outcome {
        WorthQueryApplicationCommitOutcome::Committed(commit) => {
            WorthQueryElevationRequestOutcome::Requested(WorthQueryRequestedElevation::new(
                binding, commit,
            ))
        }
        WorthQueryApplicationCommitOutcome::AlreadyCommitted(commit) => {
            WorthQueryElevationRequestOutcome::AlreadyRequested(WorthQueryRequestedElevation::new(
                binding, commit,
            ))
        }
        WorthQueryApplicationCommitOutcome::Stale(stale) => {
            WorthQueryElevationRequestOutcome::Stale(stale)
        }
        WorthQueryApplicationCommitOutcome::ProductStale(stale) => {
            WorthQueryElevationRequestOutcome::ProductStale(stale)
        }
        WorthQueryApplicationCommitOutcome::Cancelled => {
            WorthQueryElevationRequestOutcome::Cancelled
        }
        WorthQueryApplicationCommitOutcome::TimedOut => WorthQueryElevationRequestOutcome::TimedOut,
        WorthQueryApplicationCommitOutcome::Denied(denial) => {
            WorthQueryElevationRequestOutcome::Denied(denial)
        }
        WorthQueryApplicationCommitOutcome::Aborted => WorthQueryElevationRequestOutcome::Aborted,
        WorthQueryApplicationCommitOutcome::Deferred(deferred) => {
            WorthQueryElevationRequestOutcome::Deferred(deferred)
        }
        WorthQueryApplicationCommitOutcome::ProductUnpublished(unpublished) => {
            WorthQueryElevationRequestOutcome::ProductUnpublished(unpublished)
        }
        WorthQueryApplicationCommitOutcome::SettlementDeferred(deferred) => {
            WorthQueryElevationRequestOutcome::SettlementDeferred(deferred)
        }
        WorthQueryApplicationCommitOutcome::Indeterminate(_) => {
            WorthQueryElevationRequestOutcome::Indeterminate
        }
    }
}
