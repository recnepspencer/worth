//! Native causal reporting through the actual installed Bridge owner.
use super::WorthQueryInstalledApplicationBridge;
use worth_runtime_bridge::facade::{
    BridgeCausalEnvelopeConstructionStop, BridgeCausalExplanationEnvelope,
    BridgeConditionalCausalEnvelopeRequest, BridgeConditionalDenial,
};

#[derive(Debug)]
pub enum WorthQueryBridgeCausalReportingDenial {
    Owner(BridgeConditionalDenial),
    Envelope(BridgeCausalEnvelopeConstructionStop),
}

impl WorthQueryInstalledApplicationBridge {
    /// Lower-owner reporting only; no fresh Query read or disclosure is admitted.
    pub fn assemble_native_causal_report(
        &self,
        request: BridgeConditionalCausalEnvelopeRequest,
    ) -> Result<BridgeCausalExplanationEnvelope, WorthQueryBridgeCausalReportingDenial> {
        let owner = self.conditional();
        owner
            .memory()
            .map_err(WorthQueryBridgeCausalReportingDenial::Owner)?
            .assemble_conditional_causal_envelope(request)
            .map_err(WorthQueryBridgeCausalReportingDenial::Envelope)
    }
}
