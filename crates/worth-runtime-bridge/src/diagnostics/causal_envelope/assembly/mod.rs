use super::{
    BridgeCausalEnvelopeDenial, BridgeCausalEnvelopeDenialKind, BridgeCausalEvidenceFamily,
    BridgeCausalEvidenceOwner, BridgeCausalExplanationEnvelope,
};
use crate::diagnostics::BridgeDiagnosticsFacade;

mod native_evidence;
mod native_request;
mod progress;
mod request;
mod source;

pub(crate) use native_request::assemble_native;
pub use native_request::BridgeConditionalCausalEnvelopeRequest;
pub use request::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalInspectionAdmissionSummary,
    BridgeCausalInspectionAdmissionSummaryKind,
};

impl BridgeDiagnosticsFacade {
    pub fn assemble_causal_explanation_envelope(
        &self,
        request: BridgeCausalEnvelopeAssemblyRequest,
    ) -> Result<BridgeCausalExplanationEnvelope, BridgeCausalEnvelopeDenial> {
        assemble(request, source::SourceContext::LegacyDiagnostics(self))
    }
}

fn assemble(
    request: BridgeCausalEnvelopeAssemblyRequest,
    source: source::SourceContext<'_>,
) -> Result<BridgeCausalExplanationEnvelope, BridgeCausalEnvelopeDenial> {
    let mut progress =
        progress::BridgeCausalEnvelopeAssemblyProgress::for_request(&request, source.is_native());
    for reference in request.references() {
        progress.bind_reference(&source, reference)?;
    }
    if !source.is_native() && !progress.has_required_bridge_route_evidence() {
        return Err(BridgeCausalEnvelopeDenial::new(
            BridgeCausalEnvelopeDenialKind::MissingRequiredBridgeRouteEvidence,
            BridgeCausalEvidenceFamily::BridgeRoute,
            BridgeCausalEvidenceOwner::RuntimeBridge,
            BridgeCausalEvidenceOwner::RuntimeBridge,
            request.request_evidence_identity().clone(),
            progress.success_counters(),
        ));
    }
    let counters = progress.success_counters();
    Ok(BridgeCausalExplanationEnvelope::new(
        request,
        progress.into_bindings(),
        counters,
    ))
}
