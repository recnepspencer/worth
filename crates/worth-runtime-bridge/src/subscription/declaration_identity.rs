use crate::identity::{
    AdmittedSubscriptionIdentityTag, BridgeIdentity,
    SubscriptionAcknowledgementFrontierIdentityTag, SubscriptionActiveIdentityTag,
    SubscriptionAdmittedResumeBasisIdentityTag, SubscriptionBasisIdentityTag,
    SubscriptionCheckpointIdentityTag, SubscriptionCheckpointReadyIdentityTag,
    SubscriptionConsumerContractIdentityTag, SubscriptionContinuationCandidateIdentityTag,
    SubscriptionContinuationChildIdentityTag, SubscriptionContinuationDecisionIdentityTag,
    SubscriptionContinuationIndexIdentityTag, SubscriptionDeclarationFamilyIdentityTag,
    SubscriptionDeclarationIdentityTag, SubscriptionDeliveryBufferLifecycleIdentityTag,
    SubscriptionDeliveryContentDigestTag, SubscriptionDeliveryCostProfileIdentityTag,
    SubscriptionDeliveryDiagnosticsReferenceIdentityTag, SubscriptionDeliveryFamilyIdentityTag,
    SubscriptionDeliveryMemberIdentityTag, SubscriptionDeliveryReplayPlanIdentityTag,
    SubscriptionDeliveryReplayReadinessIdentityTag, SubscriptionDeliveryWindowIdentityTag,
    SubscriptionDeliveryWindowOpenIdentityTag, SubscriptionDuplicateReplayPolicyIdentityTag,
    SubscriptionFamilyRegistryIdentityTag, SubscriptionFanoutConsumerBindingIdentityTag,
    SubscriptionFanoutDeliveryProjectionIdentityTag,
    SubscriptionFanoutDeliveryProjectionSetIdentityTag, SubscriptionFanoutLayoutIdentityTag,
    SubscriptionFanoutPlanIdentityTag, SubscriptionFanoutProjectionValidationIdentityTag,
    SubscriptionHistoricalPreviousValueEvidenceIdentityTag,
    SubscriptionHistoricalTemporalReadinessIdentityTag,
    SubscriptionHistoricalTemporalReplayBasisIdentityTag,
    SubscriptionHistoricalTemporalReplayRequestIdentityTag,
    SubscriptionHistoricalTruthBasisIdentityTag, SubscriptionLifecycleIdentityTag,
    SubscriptionMixedCauseDeliveryWindowIdentityTag, SubscriptionMixedCauseDeniedCauseIdentityTag,
    SubscriptionMixedCauseOrderedCauseIdentityTag, SubscriptionMixedCauseOrderingIdentityTag,
    SubscriptionMixedCauseOrderingRequestIdentityTag,
    SubscriptionMixedCauseSuppressedCauseIdentityTag, SubscriptionPreviewActiveIdentityTag,
    SubscriptionPreviewAuthoritativeReadmissionIdentityTag, SubscriptionPreviewBasisIdentityTag,
    SubscriptionPreviewDiscardResidueProofIdentityTag, SubscriptionPreviewLifecycleIdentityTag,
    SubscriptionPreviewLifecyclePromotionIdentityTag,
    SubscriptionPreviewLifecycleResidueEnvelopeIdentityTag,
    SubscriptionPreviewParentBasisIdentityTag, SubscriptionPreviewResidueArtifactIdentityTag,
    SubscriptionPreviewResidueScopeIdentityTag, SubscriptionPreviewResidueScopeIndexIdentityTag,
    SubscriptionPreviewScopeIdentityTag, SubscriptionPreviewTemporalActivationReadyIdentityTag,
    SubscriptionPreviewTemporalAdmissionIdentityTag, SubscriptionPreviewWorkRecordIdentityTag,
    SubscriptionPreviewWorkTraceIdentityTag, SubscriptionReplayIdentityTag,
    SubscriptionReplayReadinessIdentityTag, SubscriptionResumeAdmissionIdentityTag,
    SubscriptionResumePlanIdentityTag, SubscriptionRetainedDeliveryReplaySeedIdentityTag,
    SubscriptionRetainedDeliveryResumeBasisIdentityTag,
    SubscriptionRetainedDeliveryWindowSeedIdentityTag,
    SubscriptionRetainedInflightAsyncResumeBasisIdentityTag,
    SubscriptionRetainedResumeBasisIdentityTag, SubscriptionRetainedTemporalResumeBasisIdentityTag,
    SubscriptionSharedDeliveryAcknowledgementIdentityTag,
    SubscriptionSharedDeliveryBundleDraftIdentityTag,
    SubscriptionSharedDeliveryBundleSealedIdentityTag, SubscriptionSharedDeliveryLayoutIdentityTag,
    SubscriptionSharedDeliveryPlanIdentityTag, SubscriptionSharedDeliveryProjectionIdentityTag,
    SubscriptionSharingEligibilityIdentityTag, SubscriptionSignalStrategyIdentityTag,
    SubscriptionTemporalActivationReadyIdentityTag, SubscriptionTemporalAdmissionIdentityTag,
    SubscriptionTemporalCauseRecordIdentityTag, SubscriptionTemporalDeliveryPlanIdentityTag,
    SubscriptionTemporalWakeRoutingRequestIdentityTag,
};

pub type BridgeSubscriptionDeclarationFamilyIdentity =
    BridgeIdentity<SubscriptionDeclarationFamilyIdentityTag>;
pub type BridgeSubscriptionDeclarationIdentity = BridgeIdentity<SubscriptionDeclarationIdentityTag>;
pub type BridgeSubscriptionFamilyRegistryIdentity =
    BridgeIdentity<SubscriptionFamilyRegistryIdentityTag>;
pub type BridgeSubscriptionBasisIdentity = BridgeIdentity<SubscriptionBasisIdentityTag>;
pub type BridgeSignalStrategyIdentity = BridgeIdentity<SubscriptionSignalStrategyIdentityTag>;
pub type BridgeAdmittedSubscriptionIdentity = BridgeIdentity<AdmittedSubscriptionIdentityTag>;
pub type BridgeSubscriptionLifecycleIdentity = BridgeIdentity<SubscriptionLifecycleIdentityTag>;
pub type BridgeSubscriptionReplayIdentity = BridgeIdentity<SubscriptionReplayIdentityTag>;
pub type BridgeSubscriptionDeliveryCostProfileIdentity =
    BridgeIdentity<SubscriptionDeliveryCostProfileIdentityTag>;
pub type BridgeSubscriptionConsumerContractIdentity =
    BridgeIdentity<SubscriptionConsumerContractIdentityTag>;
pub type BridgeActiveSubscriptionIdentity = BridgeIdentity<SubscriptionActiveIdentityTag>;
pub type BridgeSubscriptionDeliveryFamilyIdentity =
    BridgeIdentity<SubscriptionDeliveryFamilyIdentityTag>;
pub type BridgeSubscriptionDeliveryWindowOpenIdentity =
    BridgeIdentity<SubscriptionDeliveryWindowOpenIdentityTag>;
pub type BridgeSubscriptionDeliveryWindowIdentity =
    BridgeIdentity<SubscriptionDeliveryWindowIdentityTag>;
pub type BridgeSubscriptionDeliveryMemberIdentity =
    BridgeIdentity<SubscriptionDeliveryMemberIdentityTag>;
pub type BridgeSubscriptionDeliveryContentDigest =
    BridgeIdentity<SubscriptionDeliveryContentDigestTag>;
pub type BridgeSubscriptionDeliveryDiagnosticsReferenceIdentity =
    BridgeIdentity<SubscriptionDeliveryDiagnosticsReferenceIdentityTag>;
pub type BridgeSubscriptionDeliveryBufferLifecycleIdentity =
    BridgeIdentity<SubscriptionDeliveryBufferLifecycleIdentityTag>;
pub type BridgeSubscriptionSharingEligibilityIdentity =
    BridgeIdentity<SubscriptionSharingEligibilityIdentityTag>;
pub type BridgeSubscriptionFanoutPlanIdentity = BridgeIdentity<SubscriptionFanoutPlanIdentityTag>;
pub type BridgeSubscriptionFanoutLayoutIdentity =
    BridgeIdentity<SubscriptionFanoutLayoutIdentityTag>;
pub type BridgeSubscriptionFanoutConsumerBindingIdentity =
    BridgeIdentity<SubscriptionFanoutConsumerBindingIdentityTag>;
pub type BridgeSubscriptionFanoutDeliveryProjectionIdentity =
    BridgeIdentity<SubscriptionFanoutDeliveryProjectionIdentityTag>;
pub type BridgeSubscriptionFanoutDeliveryProjectionSetIdentity =
    BridgeIdentity<SubscriptionFanoutDeliveryProjectionSetIdentityTag>;
pub type BridgeSubscriptionRetainedDeliveryWindowSeedIdentity =
    BridgeIdentity<SubscriptionRetainedDeliveryWindowSeedIdentityTag>;
pub type BridgeSubscriptionRetainedDeliveryReplaySeedIdentity =
    BridgeIdentity<SubscriptionRetainedDeliveryReplaySeedIdentityTag>;
pub type BridgeSubscriptionFanoutProjectionValidationIdentity =
    BridgeIdentity<SubscriptionFanoutProjectionValidationIdentityTag>;
pub type BridgeSubscriptionDeliveryReplayReadinessIdentity =
    BridgeIdentity<SubscriptionDeliveryReplayReadinessIdentityTag>;
pub type BridgeSubscriptionDeliveryReplayPlanIdentity =
    BridgeIdentity<SubscriptionDeliveryReplayPlanIdentityTag>;
pub type BridgeSubscriptionAcknowledgementFrontierIdentity =
    BridgeIdentity<SubscriptionAcknowledgementFrontierIdentityTag>;
pub type BridgeSubscriptionCheckpointReadyIdentity =
    BridgeIdentity<SubscriptionCheckpointReadyIdentityTag>;
pub type BridgeSubscriptionCheckpointIdentity = BridgeIdentity<SubscriptionCheckpointIdentityTag>;
pub type BridgeSubscriptionResumeAdmissionIdentity =
    BridgeIdentity<SubscriptionResumeAdmissionIdentityTag>;
pub type BridgeSubscriptionResumePlanIdentity = BridgeIdentity<SubscriptionResumePlanIdentityTag>;
pub type BridgeSubscriptionRetainedResumeBasisIdentity =
    BridgeIdentity<SubscriptionRetainedResumeBasisIdentityTag>;
pub type BridgeSubscriptionRetainedTemporalResumeBasisIdentity =
    BridgeIdentity<SubscriptionRetainedTemporalResumeBasisIdentityTag>;
pub type BridgeSubscriptionRetainedInflightAsyncResumeBasisIdentity =
    BridgeIdentity<SubscriptionRetainedInflightAsyncResumeBasisIdentityTag>;
pub type BridgeSubscriptionRetainedDeliveryResumeBasisIdentity =
    BridgeIdentity<SubscriptionRetainedDeliveryResumeBasisIdentityTag>;
pub type BridgeSubscriptionAdmittedResumeBasisIdentity =
    BridgeIdentity<SubscriptionAdmittedResumeBasisIdentityTag>;
pub type BridgeSubscriptionReplayReadinessIdentity =
    BridgeIdentity<SubscriptionReplayReadinessIdentityTag>;
pub type BridgeSubscriptionMixedCauseOrderingRequestIdentity =
    BridgeIdentity<SubscriptionMixedCauseOrderingRequestIdentityTag>;
pub type BridgeSubscriptionMixedCauseOrderedCauseIdentity =
    BridgeIdentity<SubscriptionMixedCauseOrderedCauseIdentityTag>;
pub type BridgeSubscriptionMixedCauseSuppressedCauseIdentity =
    BridgeIdentity<SubscriptionMixedCauseSuppressedCauseIdentityTag>;
pub type BridgeSubscriptionMixedCauseDeniedCauseIdentity =
    BridgeIdentity<SubscriptionMixedCauseDeniedCauseIdentityTag>;
pub type BridgeSubscriptionMixedCauseOrderingIdentity =
    BridgeIdentity<SubscriptionMixedCauseOrderingIdentityTag>;
pub type BridgeSubscriptionMixedCauseDeliveryWindowIdentity =
    BridgeIdentity<SubscriptionMixedCauseDeliveryWindowIdentityTag>;
pub type BridgeSubscriptionSharedDeliveryPlanIdentity =
    BridgeIdentity<SubscriptionSharedDeliveryPlanIdentityTag>;
pub type BridgeSubscriptionSharedDeliveryLayoutIdentity =
    BridgeIdentity<SubscriptionSharedDeliveryLayoutIdentityTag>;
pub type BridgeSubscriptionSharedDeliveryBundleDraftIdentity =
    BridgeIdentity<SubscriptionSharedDeliveryBundleDraftIdentityTag>;
pub type BridgeSubscriptionSharedDeliveryBundleSealedIdentity =
    BridgeIdentity<SubscriptionSharedDeliveryBundleSealedIdentityTag>;
pub type BridgeSubscriptionSharedDeliveryProjectionIdentity =
    BridgeIdentity<SubscriptionSharedDeliveryProjectionIdentityTag>;
pub type BridgeSubscriptionSharedDeliveryAcknowledgementIdentity =
    BridgeIdentity<SubscriptionSharedDeliveryAcknowledgementIdentityTag>;
pub type BridgeSubscriptionDuplicateReplayPolicyIdentity =
    BridgeIdentity<SubscriptionDuplicateReplayPolicyIdentityTag>;
pub type BridgeSubscriptionContinuationIndexIdentity =
    BridgeIdentity<SubscriptionContinuationIndexIdentityTag>;
pub type BridgeSubscriptionContinuationCandidateIdentity =
    BridgeIdentity<SubscriptionContinuationCandidateIdentityTag>;
pub type BridgeSubscriptionContinuationDecisionIdentity =
    BridgeIdentity<SubscriptionContinuationDecisionIdentityTag>;
pub type BridgeSubscriptionContinuationChildIdentity =
    BridgeIdentity<SubscriptionContinuationChildIdentityTag>;
pub type BridgeSubscriptionPreviewBasisIdentity =
    BridgeIdentity<SubscriptionPreviewBasisIdentityTag>;
pub type BridgePreviewActiveSubscriptionIdentity =
    BridgeIdentity<SubscriptionPreviewActiveIdentityTag>;
pub type BridgeSubscriptionPreviewScopeIdentity =
    BridgeIdentity<SubscriptionPreviewScopeIdentityTag>;
pub type BridgeSubscriptionPreviewParentBasisIdentity =
    BridgeIdentity<SubscriptionPreviewParentBasisIdentityTag>;
pub type BridgeSubscriptionPreviewLifecycleIdentity =
    BridgeIdentity<SubscriptionPreviewLifecycleIdentityTag>;
pub type BridgeSubscriptionPreviewResidueScopeIdentity =
    BridgeIdentity<SubscriptionPreviewResidueScopeIdentityTag>;
pub type BridgeSubscriptionPreviewResidueScopeIndexIdentity =
    BridgeIdentity<SubscriptionPreviewResidueScopeIndexIdentityTag>;
pub type BridgeSubscriptionPreviewResidueArtifactIdentity =
    BridgeIdentity<SubscriptionPreviewResidueArtifactIdentityTag>;
pub type BridgeSubscriptionPreviewLifecycleResidueEnvelopeIdentity =
    BridgeIdentity<SubscriptionPreviewLifecycleResidueEnvelopeIdentityTag>;
pub type BridgeSubscriptionPreviewDiscardResidueProofIdentity =
    BridgeIdentity<SubscriptionPreviewDiscardResidueProofIdentityTag>;
pub type BridgeSubscriptionPreviewLifecyclePromotionIdentity =
    BridgeIdentity<SubscriptionPreviewLifecyclePromotionIdentityTag>;
pub type BridgeSubscriptionPreviewAuthoritativeReadmissionIdentity =
    BridgeIdentity<SubscriptionPreviewAuthoritativeReadmissionIdentityTag>;
pub type BridgeSubscriptionPreviewWorkTraceIdentity =
    BridgeIdentity<SubscriptionPreviewWorkTraceIdentityTag>;
pub type BridgeSubscriptionPreviewWorkRecordIdentity =
    BridgeIdentity<SubscriptionPreviewWorkRecordIdentityTag>;
pub type BridgeSubscriptionTemporalAdmissionIdentity =
    BridgeIdentity<SubscriptionTemporalAdmissionIdentityTag>;
pub type BridgeSubscriptionTemporalActivationReadyIdentity =
    BridgeIdentity<SubscriptionTemporalActivationReadyIdentityTag>;
pub type BridgeSubscriptionTemporalWakeRoutingRequestIdentity =
    BridgeIdentity<SubscriptionTemporalWakeRoutingRequestIdentityTag>;
pub type BridgeSubscriptionHistoricalTruthBasisIdentity =
    BridgeIdentity<SubscriptionHistoricalTruthBasisIdentityTag>;
pub type BridgeSubscriptionHistoricalPreviousValueEvidenceIdentity =
    BridgeIdentity<SubscriptionHistoricalPreviousValueEvidenceIdentityTag>;
pub type BridgeSubscriptionHistoricalTemporalReplayBasisIdentity =
    BridgeIdentity<SubscriptionHistoricalTemporalReplayBasisIdentityTag>;
pub type BridgeSubscriptionHistoricalTemporalReplayRequestIdentity =
    BridgeIdentity<SubscriptionHistoricalTemporalReplayRequestIdentityTag>;
pub type BridgeSubscriptionHistoricalTemporalReadinessIdentity =
    BridgeIdentity<SubscriptionHistoricalTemporalReadinessIdentityTag>;
pub type BridgeSubscriptionTemporalCauseRecordIdentity =
    BridgeIdentity<SubscriptionTemporalCauseRecordIdentityTag>;
pub type BridgeSubscriptionTemporalDeliveryPlanIdentity =
    BridgeIdentity<SubscriptionTemporalDeliveryPlanIdentityTag>;
pub type BridgeSubscriptionPreviewTemporalAdmissionIdentity =
    BridgeIdentity<SubscriptionPreviewTemporalAdmissionIdentityTag>;
pub type BridgeSubscriptionPreviewTemporalActivationReadyIdentity =
    BridgeIdentity<SubscriptionPreviewTemporalActivationReadyIdentityTag>;
