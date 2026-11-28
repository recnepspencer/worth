use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::Arc;

use worth_foundational::facade::FoundationalIdentityKind;

use crate::clone_budget::CheapClone;
use crate::identity_authority::{
    admit_bridge_truth_authority_identity, admit_bridge_truth_authority_identity_for_kind,
    BridgeEvidenceReferenceIdentityKind, BridgePreviewSessionIdentityKind,
    BridgeTruthAuthorityIdentity, BridgeTruthBoundaryBridgedIdentity,
    BridgeWritebackDeclarationIdentityKind,
};

mod evidence;
pub(crate) mod lineage;
pub(crate) mod record;
pub(crate) mod truth;

pub use evidence::{bridge_identity_reporting_label, BridgeIdentityEvidence};

pub struct BridgeIdentity<Tag> {
    value: Arc<str>,
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
            _tag: PhantomData,
        }
    }

    pub(crate) fn admit_bridge_owned(value: impl Into<Arc<str>>) -> Self {
        Self::new(admit_bridge_truth_authority_identity(value))
    }

    pub(crate) fn as_str(&self) -> &str {
        self.value.as_ref()
    }

    pub(crate) fn from_reference_evidence(
        evidence: BridgeTruthBoundaryBridgedIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind>,
    ) -> Self {
        Self {
            value: evidence.value().clone(),
            _tag: PhantomData,
        }
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
        admit_bridge_truth_authority_identity_for_kind::<_, Tag::Kind>(Arc::clone(&self.value))
            .bridge_trust_boundary()
    }

    pub fn terminal_projection_for_reporting(&self) -> &str {
        self.as_str()
    }
}

impl<Tag> Clone for BridgeIdentity<Tag> {
    fn clone(&self) -> Self {
        Self {
            value: Arc::clone(&self.value),
            _tag: PhantomData,
        }
    }
}

impl<Tag> CheapClone for BridgeIdentity<Tag> {}

macro_rules! bridge_identity_tags {
    ($($tag:ident),+ $(,)?) => {
        $(pub enum $tag {})+
    };
}

bridge_identity_tags! {
    CommittedPatchDigestTag,
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
    SubscriptionPreviewDiscardResidueProofIdentityTag,
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
    ContinuityIdentityTag,
    HistoricalEvaluationRecordIdentityTag, HistoricalEvaluationDecisionLogIdentityTag,
    HistoricalEvaluationArtifactIdentityTag, HistoricalEvaluationFailureIdentityTag, WorkloadIdentityTag,
    BulkPlanningIdentityTag, BulkAdmissionProfileIdentityTag, BulkPacketRegionIdentityTag,
    BulkContinuityMemberIdentityTag, BulkTruthViewMemberIdentityTag, BulkWorkloadSegmentIdentityTag,
    ReducedPublicationIdentityTag, ReducedRoutingTargetIdentityTag, ReducedTruthViewIdentityTag,
    ReducedContinuityIdentityTag, ReducedWideningIdentityTag, RoutingPacketIdentityTag,
    TruthViewPacketIdentityTag, ContinuityPacketIdentityTag, WideningPacketIdentityTag,
    ReductionPacketIdentityTag,


    SourceDeclarationIdentityTag, SourceContractIdentityTag, SourceMaterializationRecordIdentityTag,
    SourceFailureRecordIdentityTag,
    PreviewSessionIdentityTag,

    TemporalBasisIdentityTag,
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
    WritebackDeclarationIdentityTag,
    WritebackContractIdentityTag,
    WritebackCausalityIdentityTag,

    WritebackEffectIdentityTag, WritebackIdempotenceIdentityTag, WritebackLoopPreventionIdentityTag,


}

impl BridgeIdentityAuthorityKind for PreviewSessionIdentityTag {
    type Kind = BridgePreviewSessionIdentityKind;
}

impl BridgeIdentityAuthorityKind for WritebackDeclarationIdentityTag {
    type Kind = BridgeWritebackDeclarationIdentityKind;
}

impl<Tag> PartialEq for BridgeIdentity<Tag> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
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
        self.value.cmp(&other.value)
    }
}

impl<Tag> Hash for BridgeIdentity<Tag> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state)
    }
}

impl<Tag> fmt::Debug for BridgeIdentity<Tag> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BridgeIdentity").field(&"<opaque>").finish()
    }
}
