use super::{BridgeCausalEnvelopeDenial, BridgeCausalExplanationEnvelope};
use crate::diagnostics::BridgeDiagnosticsFacade;

mod native_evidence;
mod native_request;
mod progress;
mod query_observation;
mod request;
mod source;

pub(crate) use native_request::assemble_native;
pub use native_request::BridgeConditionalCausalEnvelopeRequest;
pub use query_observation::{
    BridgeQueryObservationReportingInputDenial, BridgeQueryObservationReportingRequest,
    BridgeQueryObservationReportingStop,
};
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

fn assemble<Source: source::CausalEnvelopeAssemblySource>(
    request: BridgeCausalEnvelopeAssemblyRequest,
    mut source: Source,
) -> Result<BridgeCausalExplanationEnvelope, Source::Stop> {
    let mut progress = source.prepare_progress(&request)?;
    for reference in request.references() {
        source.bind_reference(&mut progress, reference)?;
    }
    if source.requires_bridge_route() && !progress.has_required_bridge_route_evidence() {
        return Err(source.missing_bridge_route(&request, &progress));
    }
    source.finish(request, progress)
}
