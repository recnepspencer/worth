//! A genuine observation is opaque until the sealed owner verifies its affinity.
use super::super::{
    BridgeCausalEnvelopeCounters, BridgeCausalEnvelopeDenial, BridgeCausalEnvelopeDenialKind,
    BridgeCausalEvidenceFamily, BridgeCausalEvidenceOwner, BridgeCausalExplanationEnvelope,
};
use super::{BridgeCausalEnvelopeAssemblyRequest, BridgeCausalInspectionAdmissionSummary};
use crate::conditional_execution::{BridgeConditionalCausalObservation, BridgeOwnedSignalRuntime};

/// A lower-runtime reporting request, never fresh Query admission authority.
/// Construction retains the actual observation without projecting its body.
pub struct BridgeConditionalCausalEnvelopeRequest {
    summary: BridgeCausalInspectionAdmissionSummary,
    observation: BridgeConditionalCausalObservation,
}

impl BridgeConditionalCausalEnvelopeRequest {
    pub fn new(
        summary: BridgeCausalInspectionAdmissionSummary,
        observation: BridgeConditionalCausalObservation,
    ) -> Result<Self, BridgeCausalEnvelopeDenial> {
        if summary.requires_application_inspection_admission() {
            return Err(denial(
                &summary,
                BridgeCausalEnvelopeDenialKind::ApplicationInspectionAdmissionRequired,
            ));
        }
        if summary.expected_query_observation_identity().is_none() {
            return Err(denial(
                &summary,
                BridgeCausalEnvelopeDenialKind::QueryObservationAssociationMismatch,
            ));
        }
        Ok(Self {
            summary,
            observation,
        })
    }
}

pub(crate) fn assemble_native(
    owner: &BridgeOwnedSignalRuntime,
    request: BridgeConditionalCausalEnvelopeRequest,
) -> Result<BridgeCausalExplanationEnvelope, BridgeCausalEnvelopeDenial> {
    if !request.observation.belongs_to(owner) {
        return Err(denial(
            &request.summary,
            BridgeCausalEnvelopeDenialKind::ForeignNativeObservation,
        ));
    }
    // No Native projection, receipt access or reference derivation precedes affinity.
    let evidence = super::native_evidence::NativeEvidence::from_verified(&request.observation)?;
    let expected = request
        .summary
        .expected_query_observation_identity()
        .unwrap()
        .clone();
    let references = evidence.references_with_query(expected)?;
    let descriptor =
        BridgeCausalEnvelopeAssemblyRequest::from_query_admission(request.summary, references)?;
    let envelope = super::assemble(descriptor, super::source::SourceContext::Native(&evidence))?;
    Ok(envelope.retain_native(request.observation))
}

fn denial(
    summary: &BridgeCausalInspectionAdmissionSummary,
    kind: BridgeCausalEnvelopeDenialKind,
) -> BridgeCausalEnvelopeDenial {
    BridgeCausalEnvelopeDenial::new(
        kind,
        BridgeCausalEvidenceFamily::BridgeConditionalObservation,
        BridgeCausalEvidenceOwner::RuntimeBridge,
        BridgeCausalEvidenceOwner::RuntimeBridge,
        summary.summary_evidence_identity().clone(),
        BridgeCausalEnvelopeCounters::empty(),
    )
}
