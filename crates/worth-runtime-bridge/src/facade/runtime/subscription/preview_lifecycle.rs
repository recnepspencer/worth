use super::*;

impl RuntimeBridge {
    /// Captures one sealed preview lifecycle residue envelope from explicit
    /// preview-local residue inputs and a matching preview work trace.
    pub fn capture_preview_lifecycle_residue_envelope(
        &self,
        preview_active: &BridgePreviewActiveSubscription,
        preview_work_trace: &BridgeSubscriptionPreviewWorkTrace,
        residue_inputs: Vec<BridgeSubscriptionPreviewLifecycleResidueInput>,
    ) -> Result<
        BridgeSubscriptionPreviewLifecycleResidueEnvelope,
        BridgeSubscriptionPreviewLifecycleResidueEnvelopeRejection,
    > {
        let _ = self;
        BridgeSubscriptionPreviewLifecycleResidueEnvelope::capture(
            preview_active,
            preview_work_trace,
            residue_inputs,
        )
    }

    /// Admits preview discard only after a sealed lifecycle residue envelope
    /// proves zero residue across the required preview-local lifecycle lanes.
    pub fn admit_preview_lifecycle_discard(
        &self,
        preview_active: BridgePreviewActiveSubscription,
        residue_envelope: BridgeSubscriptionPreviewLifecycleResidueEnvelope,
    ) -> Result<
        BridgeSubscriptionPreviewLifecycleDiscardProof,
        (
            BridgePreviewActiveSubscription,
            BridgeSubscriptionPreviewLifecycleDiscardRejection,
        ),
    > {
        let _ = self;
        BridgeSubscriptionPreviewLifecycleDiscardProof::prove(preview_active, residue_envelope)
    }

    /// Admits preview promotion as a preview-local lifecycle boundary before
    /// authoritative readmission occurs.
    pub fn admit_preview_lifecycle_promotion(
        &self,
        preview_active: &BridgePreviewActiveSubscription,
        preview_work_trace: &BridgeSubscriptionPreviewWorkTrace,
        residue_envelope: &BridgeSubscriptionPreviewLifecycleResidueEnvelope,
        promotion_record: &BridgePreviewPromotionRecord,
    ) -> Result<
        BridgeSubscriptionPreviewLifecyclePromotion,
        BridgeSubscriptionPreviewLifecyclePromotionRejection,
    > {
        let _ = self;
        BridgeSubscriptionPreviewLifecyclePromotion::admit(
            preview_active,
            preview_work_trace,
            residue_envelope,
            promotion_record,
        )
    }

    /// Re-admits authoritative subscription lifecycle proof from a previously
    /// admitted preview promotion boundary.
    pub fn prepare_authoritative_preview_readmission(
        &self,
        promotion: BridgeSubscriptionPreviewLifecyclePromotion,
        promoted_activation_ready: &BridgeSubscriptionActivationReady,
    ) -> Result<
        BridgeSubscriptionAuthoritativePreviewReadmission,
        (
            BridgeSubscriptionPreviewLifecyclePromotion,
            BridgeSubscriptionAuthoritativePreviewReadmissionRejection,
        ),
    > {
        let _ = self;
        BridgeSubscriptionAuthoritativePreviewReadmission::prepare(
            promotion,
            promoted_activation_ready,
        )
    }
}
