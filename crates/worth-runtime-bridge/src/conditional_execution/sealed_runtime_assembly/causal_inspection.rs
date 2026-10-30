use super::BridgeSealedRuntimeAssembly;
use crate::diagnostics::causal_envelope::{
    BridgeCausalEnvelopeDenial, BridgeCausalExplanationEnvelope,
    BridgeConditionalCausalEnvelopeRequest,
};

impl BridgeSealedRuntimeAssembly {
    /// Assembles lower-runtime evidence after checking this exact sealed owner.
    /// This reporting operation does not admit a fresh Query read or disclosure.
    pub fn assemble_conditional_causal_envelope(
        &self,
        request: BridgeConditionalCausalEnvelopeRequest,
    ) -> Result<BridgeCausalExplanationEnvelope, BridgeCausalEnvelopeDenial> {
        crate::diagnostics::causal_envelope::assembly::assemble_native(&self.runtime, request)
    }
}

impl BridgeSealedRuntimeAssembly {
    #[doc(hidden)]
    pub fn configure_conditional_history_for_test(&self, enabled: bool, limit: usize) -> bool {
        self.runtime
            .bridge
            .diagnostics
            .configure_conditional_history_for_test(enabled, limit)
    }
}
