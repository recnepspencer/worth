use super::*;

impl RuntimeBridge {
    /// Specialist validation entrypoint for writeback declarations.
    ///
    /// Everyday bridge flows should reach writeback through higher-level
    /// promotion or authority workflows, not by assembling declarations by hand.
    pub fn validate_writeback_declaration(
        &self,
        declaration: BridgeWritebackDeclaration,
    ) -> Result<ValidatedBridgeWritebackDeclaration, BridgeWritebackError> {
        ValidatedBridgeWritebackDeclaration::new(declaration)
    }

    /// Admits one writeback declaration against a lowered runtime policy.
    pub fn admit_writeback_declaration(
        &self,
        declaration: BridgeWritebackDeclaration,
        lowered_policy: &LoweredBridgeExecutionPolicy,
    ) -> Result<AdmittedBridgeWritebackContract, BridgeWritebackError> {
        let validated = self.validate_writeback_declaration(declaration)?;
        let authority_inputs = BridgeWritebackAuthorityInputs::new(
            self.policy.allow_replay_artifacts(),
            self.policy.diagnostics_tier(),
        );
        AdmittedBridgeWritebackContract::new(validated, authority_inputs, lowered_policy)
    }

    /// Lowers a writeback effect from contract, causality, and effect identity inputs.
    pub fn lower_writeback_effect(
        &self,
        contract: &AdmittedBridgeWritebackContract,
        causality: &BridgeWritebackNativeCausalityInputs,
        effect_identity: BridgeWritebackEffectIdentity,
        effect_intent: BridgeWritebackEffectIntent,
    ) -> BridgeDerivedWritebackEffect {
        BridgeDerivedWritebackEffect::new(effect_identity, contract, causality, effect_intent)
    }

    /// Derives feedback provenance for a lowered writeback effect.
    pub fn derive_writeback_feedback_provenance(
        &self,
        effect: &BridgeDerivedWritebackEffect,
    ) -> BridgeWritebackFeedbackProvenance {
        BridgeWritebackFeedbackProvenance::new(effect)
    }
}
