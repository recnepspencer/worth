use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::Arc;

use worth_foundational::facade::FoundationalIdentityKind;

use crate::clone_budget::CheapClone;
use crate::identity_authority::{
    admit_bridge_truth_authority_identity, admit_bridge_truth_authority_identity_for_kind,
    BridgeEvidenceReferenceIdentityKind, BridgePreviewExecutionRecordIdentityKind,
    BridgePreviewSessionDeclarationIdentityKind, BridgePreviewSessionIdentityKind,
    BridgeTruthAuthorityIdentity, BridgeTruthBoundaryBridgedIdentity,
    BridgeWritebackDeclarationIdentityKind,
};

mod evidence;

pub use evidence::{bridge_identity_reporting_label, BridgeIdentityEvidence};

pub struct BridgeIdentity<Tag> {
    value: Arc<str>,
    payload: BridgeIdentityPayload,
    _tag: PhantomData<Tag>,
}

pub trait BridgeIdentityAuthorityKind {
    type Kind: FoundationalIdentityKind;
}

impl<Tag> BridgeIdentity<Tag> {
    pub(crate) fn new(
        identity: BridgeTruthAuthorityIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind>,
    ) -> Self {
        Self {
            value: identity.value().clone(),
            payload: BridgeIdentityPayload::OpaqueText,
            _tag: PhantomData,
        }
    }

    pub(crate) fn admit_bridge_owned(value: impl Into<Arc<str>>) -> Self {
        Self::new(admit_bridge_truth_authority_identity(value))
    }

    pub(crate) fn with_payload(value: impl Into<Arc<str>>, payload: BridgeIdentityPayload) -> Self {
        Self {
            value: value.into(),
            payload,
            _tag: PhantomData,
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        self.value.as_ref()
    }

    pub(crate) fn payload(&self) -> &BridgeIdentityPayload {
        &self.payload
    }

    pub(crate) fn from_reference_evidence(
        evidence: BridgeTruthBoundaryBridgedIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind>,
    ) -> Self {
        Self {
            value: evidence.value().clone(),
            payload: BridgeIdentityPayload::OpaqueText,
            _tag: PhantomData,
        }
    }

    pub(crate) fn from_retained_evidence_reference(evidence: &BridgeIdentityEvidence) -> Self {
        Self::from_reference_evidence(evidence.revalidate_bridge_retained_reference())
    }

    pub fn bridge_admission_evidence(&self) -> BridgeIdentityEvidence {
        BridgeIdentityEvidence::from_external_authority(
            crate::identity_authority::bridge_truth_external_identity_token(Arc::clone(
                &self.value,
            )),
        )
    }
}

impl<Tag> BridgeIdentity<Tag>
where
    Tag: BridgeIdentityAuthorityKind,
{
    pub fn bridge_trust_boundary(&self) -> BridgeTruthBoundaryBridgedIdentity<Arc<str>, Tag::Kind> {
        admit_bridge_truth_authority_identity_for_kind::<Tag::Kind>(Arc::clone(&self.value))
            .bridge_trust_boundary()
    }

    pub fn terminal_projection_for_reporting(&self) -> &str {
        self.as_str()
    }
}

impl<Tag> Clone for BridgeIdentity<Tag> {
    fn clone(&self) -> Self {
        Self::with_payload(Arc::clone(&self.value), self.payload.clone())
    }
}

impl<Tag> CheapClone for BridgeIdentity<Tag> {}

macro_rules! bridge_identity_tags {
    ($($tag:ident),+ $(,)?) => {
        $(pub enum $tag {})+
    };
}

bridge_identity_tags! {
    TruthCommitTag, TruthPatchTag, TruthBranchTag, TruthSnapshotTag, CommittedPatchDigestTag,
    MappingIdTag, FrozenMappingRegistrationIdentityTag, AspectRegistrationIdTag,
    SignalInvalidationScopeTag, TruthDeltaSurfaceIdentityTag, TruthViewSelectorIdentityTag,
    HistoricalEvaluationDeclarationIdentityTag, SnapshotReadTargetIdentityTag, RouteIdentityTag,
    InvalidationIdentityTag, InvalidationTargetIdentityTag, SubscriptionSliceIdentityTag,
    SubscriptionDeclarationFamilyIdentityTag, SubscriptionDeclarationIdentityTag,
    SubscriptionSliceTargetIdentityTag, SubscriptionFamilyRegistryIdentityTag,
    SubscriptionBasisIdentityTag, SubscriptionSignalStrategyIdentityTag,
    AdmittedSubscriptionIdentityTag, SubscriptionLifecycleIdentityTag, SubscriptionReplayIdentityTag,
    SubscriptionDeliveryCostProfileIdentityTag, SubscriptionConsumerContractIdentityTag,
    SubscriptionActiveIdentityTag, SubscriptionDeliveryFamilyIdentityTag,
    SubscriptionDeliveryWindowOpenIdentityTag, SubscriptionDeliveryWindowIdentityTag,
    SubscriptionDeliveryMemberIdentityTag, SubscriptionDeliveryContentDigestTag,
    SubscriptionDeliveryDiagnosticsReferenceIdentityTag, SubscriptionDeliveryBufferLifecycleIdentityTag,
    SubscriptionSharingEligibilityIdentityTag, SubscriptionFanoutPlanIdentityTag,
    SubscriptionFanoutLayoutIdentityTag, SubscriptionFanoutConsumerBindingIdentityTag,
    SubscriptionFanoutDeliveryProjectionIdentityTag, SubscriptionFanoutDeliveryProjectionSetIdentityTag,
    SubscriptionRetainedDeliveryWindowSeedIdentityTag, SubscriptionRetainedDeliveryReplaySeedIdentityTag,
    SubscriptionFanoutProjectionValidationIdentityTag, SubscriptionDeliveryReplayReadinessIdentityTag,
    SubscriptionDeliveryReplayPlanIdentityTag, SubscriptionAcknowledgementFrontierIdentityTag,
    SubscriptionCheckpointReadyIdentityTag, SubscriptionCheckpointIdentityTag,
    SubscriptionResumeAdmissionIdentityTag, SubscriptionResumePlanIdentityTag,
    SubscriptionRetainedResumeBasisIdentityTag, SubscriptionRetainedTemporalResumeBasisIdentityTag,
    SubscriptionRetainedInflightAsyncResumeBasisIdentityTag, SubscriptionRetainedDeliveryResumeBasisIdentityTag,
    SubscriptionAdmittedResumeBasisIdentityTag, SubscriptionReplayReadinessIdentityTag,
    SubscriptionDuplicateReplayPolicyIdentityTag, SubscriptionContinuationIndexIdentityTag,
    SubscriptionContinuationCandidateIdentityTag, SubscriptionContinuationDecisionIdentityTag,
    SubscriptionContinuationChildIdentityTag, SubscriptionPreviewBasisIdentityTag,
    SubscriptionPreviewActiveIdentityTag, SubscriptionPreviewScopeIdentityTag,
    SubscriptionPreviewParentBasisIdentityTag, SubscriptionPreviewLifecycleIdentityTag,
    SubscriptionPreviewResidueScopeIdentityTag, SubscriptionPreviewResidueScopeIndexIdentityTag,
    SubscriptionPreviewResidueArtifactIdentityTag, SubscriptionPreviewLifecycleResidueEnvelopeIdentityTag,
    SubscriptionPreviewDiscardResidueProofIdentityTag, SubscriptionPreviewPromotionRecordIdentityTag,
    SubscriptionPreviewLifecyclePromotionIdentityTag, SubscriptionPreviewAuthoritativeReadmissionIdentityTag,
    SubscriptionPreviewWorkTraceIdentityTag, SubscriptionPreviewWorkRecordIdentityTag,
    SubscriptionTemporalAdmissionIdentityTag, SubscriptionTemporalActivationReadyIdentityTag,
    SubscriptionTemporalWakeRoutingRequestIdentityTag, SubscriptionHistoricalTruthBasisIdentityTag,
    SubscriptionHistoricalPreviousValueEvidenceIdentityTag, SubscriptionHistoricalTemporalReplayBasisIdentityTag,
    SubscriptionHistoricalTemporalReplayRequestIdentityTag, SubscriptionHistoricalTemporalReadinessIdentityTag,
    SubscriptionPreviewTemporalAdmissionIdentityTag, SubscriptionPreviewTemporalActivationReadyIdentityTag,
    SubscriptionTemporalCauseRecordIdentityTag, SubscriptionTemporalDeliveryPlanIdentityTag,
    SubscriptionMixedCauseOrderingRequestIdentityTag, SubscriptionMixedCauseOrderedCauseIdentityTag,
    SubscriptionMixedCauseSuppressedCauseIdentityTag, SubscriptionMixedCauseDeniedCauseIdentityTag,
    SubscriptionMixedCauseOrderingIdentityTag, SubscriptionMixedCauseDeliveryWindowIdentityTag,
    SubscriptionSharedDeliveryPlanIdentityTag, SubscriptionSharedDeliveryLayoutIdentityTag,
    SubscriptionSharedDeliveryBundleDraftIdentityTag, SubscriptionSharedDeliveryBundleSealedIdentityTag,
    SubscriptionSharedDeliveryProjectionIdentityTag, SubscriptionSharedDeliveryAcknowledgementIdentityTag,
    ContinuityIdentityTag, HistoricalResolvedLineageIdentityTag, HistoricalResolvedRecordIdentityTag,
    HistoricalEvaluationRecordIdentityTag, HistoricalEvaluationDecisionLogIdentityTag,
    HistoricalEvaluationArtifactIdentityTag, HistoricalEvaluationFailureIdentityTag, WorkloadIdentityTag,
    BulkPlanningIdentityTag, BulkAdmissionProfileIdentityTag, BulkPacketRegionIdentityTag,
    BulkContinuityMemberIdentityTag, BulkTruthViewMemberIdentityTag, BulkWorkloadSegmentIdentityTag,
    ReducedPublicationIdentityTag, ReducedRoutingTargetIdentityTag, ReducedTruthViewIdentityTag,
    ReducedContinuityIdentityTag, ReducedWideningIdentityTag, RoutingPacketIdentityTag,
    TruthViewPacketIdentityTag, ContinuityPacketIdentityTag, WideningPacketIdentityTag,
    ReductionPacketIdentityTag, ChangeStreamDeclarationIdentityTag, StreamProtocolIdentityTag,
    ConsumerContractIdentityTag, StreamMemberIdentityTag, StreamPositionIdentityTag, StreamWindowIdentityTag,
    CheckpointTokenIdentityTag, StreamReplayRecordIdentityTag, BackpressureDecisionIdentityTag,
    SourceDeclarationIdentityTag, SourceContractIdentityTag, SourceMaterializationRecordIdentityTag,
    SourceFailureRecordIdentityTag, StructuralSchemaIdentityTag, StructuralEquivalenceContractIdentityTag,
    StructuralDeclarationIdentityTag, StructuralContractIdentityTag, StructuralTruthViewBasisIdentityTag,
    MergeDeclarationIdentityTag, MergeContractIdentityTag, MergeAuthorityBasisIdentityTag,
    MergeOntologyMappingIdentityTag, MergeParentOrderIdentityTag, MergeRecordIdentityTag,
    StructuralCandidateIdentityTag, StructuralFingerprintIdentityTag, StructuralRemapRecordIdentityTag,
    StructuralBranchComparisonRecordIdentityTag, SpeculativeSignalBranchIdentityTag,
    PreviewBranchBindingIdentityTag, PreviewSessionDeclarationIdentityTag, PreviewSessionIdentityTag,
    PreviewExecutionRecordIdentityTag, PreviewDiscardRecordIdentityTag, PreviewPromotionRecordIdentityTag,
    PromotionAdmissibilityProofIdentityTag, PreviewReuseEquivalenceIdentityTag, TemporalBasisIdentityTag,
    TemporalCdcCursorIdentityTag, AsyncSourceDeclarationIdentityTag, AsyncSourceLoweringIdentityTag,
    AsyncRequestTruthViewBasisIdentityTag, AsyncRequestSubscriptionInstanceIdentityTag,
    AsyncRequestBasisBindingIdentityTag, AsyncRequestIdentityTag, AsyncInFlightRequestIdentityTag,
    AsyncCompletionEnvelopeIdentityTag, AsyncCompletionIdentityTag, AsyncCompletionReceiptIdentityTag,
    AsyncCompletionDenialIdentityTag, AsyncCompletionDenialReceiptIdentityTag,
    AsyncCompletionSupersessionIdentityTag, AsyncCompletionSupersessionReceiptIdentityTag,
    AsyncForwardCausalityIdentityTag, AsyncForwardCausalityReceiptIdentityTag,
    AsyncWritebackAdmissionIdentityTag, AsyncWritebackMapperOutputIdentityTag,
    AsyncWritebackStagedEffectIdentityTag, AsyncWritebackCommittedIdentityTag,
    AsyncWritebackNoopIdentityTag, AsyncWritebackRejectedIdentityTag,
    AsyncWritebackCausalityTransferReceiptIdentityTag, PolicyDeclarationIdentityTag,
    PolicyContractIdentityTag, LoweredExecutionPolicyIdentityTag, PolicyProvenanceIdentityTag,
    WritebackDeclarationIdentityTag, WritebackFamilyIdentityTag, WritebackStrategyIdentityTag,
    WritebackStrategyCoherenceIdentityTag, WritebackContractIdentityTag,
    WritebackAdmissionRecordIdentityTag, WritebackCausalityIdentityTag,
    WritebackMapperEnvelopeIdentityTag, WritebackMappedFamilyInputIdentityTag,
    WritebackEffectIdentityTag, WritebackIdempotenceIdentityTag, WritebackLoopPreventionIdentityTag,
    WritebackCandidateIdentityTag, WritebackMapperWitnessIdentityTag, WritebackMapperRecordIdentityTag,
    WritebackExecutionRecordIdentityTag, WritebackReplayRecordIdentityTag,
}

impl BridgeIdentityAuthorityKind for PreviewSessionIdentityTag {
    type Kind = BridgePreviewSessionIdentityKind;
}

impl BridgeIdentityAuthorityKind for PreviewSessionDeclarationIdentityTag {
    type Kind = BridgePreviewSessionDeclarationIdentityKind;
}

impl BridgeIdentityAuthorityKind for PreviewExecutionRecordIdentityTag {
    type Kind = BridgePreviewExecutionRecordIdentityKind;
}

impl BridgeIdentityAuthorityKind for WritebackDeclarationIdentityTag {
    type Kind = BridgeWritebackDeclarationIdentityKind;
}

impl<Tag> PartialEq for BridgeIdentity<Tag> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.payload, &other.payload) {
            (BridgeIdentityPayload::OpaqueText, BridgeIdentityPayload::OpaqueText) => {
                self.value == other.value
            }
            _ => self.payload == other.payload,
        }
    }
}

impl<Tag> Eq for BridgeIdentity<Tag> {}

impl<Tag> PartialOrd for BridgeIdentity<Tag> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Tag> Ord for BridgeIdentity<Tag> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.payload, &other.payload) {
            (BridgeIdentityPayload::OpaqueText, BridgeIdentityPayload::OpaqueText) => {
                self.value.cmp(&other.value)
            }
            _ => self.payload.cmp(&other.payload),
        }
    }
}

impl<Tag> Hash for BridgeIdentity<Tag> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self.payload {
            BridgeIdentityPayload::OpaqueText => self.value.hash(state),
            payload => payload.hash(state),
        }
    }
}

impl<Tag> fmt::Debug for BridgeIdentity<Tag> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BridgeIdentity").field(&"<opaque>").finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum BridgeIdentityPayload {
    OpaqueText,
    RelationalBranch { branch_id: Arc<str> },
    RelationalCommit { commit_id: u64 },
    RelationalPatch { patch_position: u64 },
    RelationalSnapshot { snapshot_id: u64, version_id: u64 },
}

#[cfg(test)]
mod tests {
    use super::{BridgeIdentity, BridgeIdentityPayload, TruthCommitTag};

    #[test]
    fn debug_does_not_expose_identity_payload() {
        let identity = BridgeIdentity::<TruthCommitTag>::admit_bridge_owned("commit-1");

        let debug = format!("{identity:?}");

        assert!(!debug.contains("commit-1"));
        assert!(debug.contains("<opaque>"));
    }

    #[test]
    fn relational_truth_identity_can_store_typed_payload() {
        let identity = BridgeIdentity::<TruthCommitTag>::with_payload(
            "relational-commit:7",
            BridgeIdentityPayload::RelationalCommit { commit_id: 7 },
        );

        assert_eq!(
            identity.payload(),
            &BridgeIdentityPayload::RelationalCommit { commit_id: 7 }
        );
    }
}
