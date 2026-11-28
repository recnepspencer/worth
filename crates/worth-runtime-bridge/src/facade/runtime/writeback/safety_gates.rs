use super::*;

impl RuntimeBridge {
    /// Computes idempotence basis data for a writeback effect under one policy.
    pub fn classify_writeback_idempotence(
        &self,
        effect: &BridgeDerivedWritebackEffect,
        lowered_policy: &LoweredBridgeExecutionPolicy,
        authoritative_state_basis: &BridgeWritebackAuthoritativeStateBasis,
        idempotence_identity: BridgeWritebackIdempotenceIdentity,
        idempotence_class: BridgeWritebackIdempotenceClass,
    ) -> BridgeWritebackIdempotenceBasis {
        BridgeWritebackIdempotenceBasis::new(
            idempotence_identity,
            effect,
            lowered_policy.digest(),
            authoritative_state_basis,
            idempotence_class,
        )
    }

    /// Classifies whether incoming feedback would create a writeback loop.
    pub fn classify_writeback_loop_prevention(
        &self,
        effect: &BridgeDerivedWritebackEffect,
        idempotence: &BridgeWritebackIdempotenceBasis,
        incoming_feedback_context: Option<&BridgeWritebackFeedbackContext>,
    ) -> BridgeWritebackLoopPreventionReport {
        BridgeWritebackLoopPreventionReport::classify(
            effect,
            idempotence,
            incoming_feedback_context,
        )
    }
}
