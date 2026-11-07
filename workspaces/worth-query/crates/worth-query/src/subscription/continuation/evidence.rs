use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity_authority::{QueryProjectionIdentity, QuerySubscriptionIdentityKind};

use super::super::active_digest::ActiveSubscriptionLaneDigest;
use super::super::delivery_dimensions::ContinuationRemapWidth;
#[cfg(test)]
use super::super::evidence_identities::{
    lifecycle_continuation_endpoint_identity, lifecycle_continuation_identity,
};
use super::super::evidence_projection::subscription_evidence_projection;
use super::super::future_selection::QuerySubscriptionFutureSelection;
use super::class::SubscriptionContinuationClass;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionContinuationEvidence {
    active_lane_digest: ActiveSubscriptionLaneDigest,
    continuation_class: SubscriptionContinuationClass,
    source_identity: WorthQueryEvidenceIdentity,
    target_identity: WorthQueryEvidenceIdentity,
    future_selection: QuerySubscriptionFutureSelection,
    basis_identity: WorthQueryEvidenceIdentity,
    checkpoint_identity: WorthQueryEvidenceIdentity,
    authority_identity: WorthQueryEvidenceIdentity,
    remap_width: ContinuationRemapWidth,
    continuation_identity: WorthQueryEvidenceIdentity,
}

impl SubscriptionContinuationEvidence {
    #[cfg(test)]
    pub(super) fn new(
        active_lane_digest: ActiveSubscriptionLaneDigest,
        continuation_class: SubscriptionContinuationClass,
        source_identity: WorthQueryEvidenceIdentity,
        target_identity: WorthQueryEvidenceIdentity,
        future_selection: QuerySubscriptionFutureSelection,
        basis_identity: WorthQueryEvidenceIdentity,
        checkpoint_identity: WorthQueryEvidenceIdentity,
        authority_identity: WorthQueryEvidenceIdentity,
        remap_width: ContinuationRemapWidth,
    ) -> Result<Self, IdentityDerivationDenial> {
        let source_identity = lifecycle_continuation_endpoint_identity("source", &source_identity)?;
        let target_identity = lifecycle_continuation_endpoint_identity("target", &target_identity)?;
        let basis_identity = lifecycle_continuation_endpoint_identity("basis", &basis_identity)?;
        let checkpoint_endpoint_identity =
            lifecycle_continuation_endpoint_identity("checkpoint", &checkpoint_identity)?;
        let authority_identity =
            lifecycle_continuation_endpoint_identity("authority", &authority_identity)?;
        let continuation_identity = lifecycle_continuation_identity(
            active_lane_digest.evidence_identity(),
            continuation_class.as_str(),
            &source_identity,
            &target_identity,
            future_selection.projection_identity(),
            &basis_identity,
            &checkpoint_endpoint_identity,
            &authority_identity,
            remap_width.get(),
        )?;
        Ok(Self {
            active_lane_digest,
            continuation_class,
            source_identity,
            target_identity,
            future_selection,
            basis_identity,
            checkpoint_identity,
            authority_identity,
            remap_width,
            continuation_identity,
        })
    }

    pub(crate) fn active_lane_digest(&self) -> &ActiveSubscriptionLaneDigest {
        &self.active_lane_digest
    }

    pub fn continuation_class(&self) -> SubscriptionContinuationClass {
        self.continuation_class
    }

    pub fn source_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.source_identity)
    }

    pub fn source_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.source_identity
    }

    pub fn target_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.target_identity)
    }

    pub fn target_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.target_identity
    }

    pub fn future_selection(&self) -> &QuerySubscriptionFutureSelection {
        &self.future_selection
    }

    pub fn basis_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.basis_identity)
    }

    pub fn basis_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_identity
    }

    pub fn checkpoint_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.checkpoint_identity)
    }

    pub fn checkpoint_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.checkpoint_identity
    }

    pub fn authority_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.authority_identity)
    }

    pub fn authority_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.authority_identity
    }

    pub fn remap_width(&self) -> u64 {
        self.remap_width.get()
    }

    pub fn continuation_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.continuation_identity)
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.continuation_identity
    }
}
