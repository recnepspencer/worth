#[cfg(test)]
use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[cfg(test)]
pub(crate) fn causal_test_compose_bridge_causal_denial_for_reporting(
    denial: &BridgeCausalEnvelopeDenial,
) -> String {
    let composed = compose_bridge_causal_denial_identity(denial)
        .expect("positive fixture identity fits canonical digest bounds");
    composed.reporting_projection().to_string()
}

#[cfg(test)]
pub(crate) fn causal_test_compose_bridge_causal_explanation_envelope_identity_for_reporting(
    envelope: &BridgeCausalExplanationEnvelope,
) -> String {
    let composed = compose_bridge_causal_explanation_envelope_identity(envelope)
        .expect("positive fixture identity fits canonical digest bounds");
    composed.reporting_projection().to_string()
}

#[cfg(test)]
pub(crate) fn causal_test_compose_bridge_causal_envelope_identity_for_reporting(
    envelope: &BridgeCausalExplanationEnvelope,
) -> String {
    let composed = compose_bridge_causal_envelope_identity(envelope.identity())
        .expect("positive fixture identity fits canonical digest bounds");
    composed.reporting_projection().to_string()
}

#[cfg(test)]
pub(crate) fn causal_test_compose_bridge_causal_envelope_receipt_identity_for_reporting(
    receipt: &BridgeCausalEnvelopeReceipt,
) -> String {
    let composed = compose_bridge_causal_envelope_receipt_identity(receipt)
        .expect("positive fixture identity fits canonical digest bounds");
    composed.reporting_projection().to_string()
}

#[cfg(test)]
pub(crate) fn causal_test_bridge_binding_reference_for_reporting(
    owner: &str,
    family: &str,
    bridge_reference: worth_runtime_bridge::facade::BridgeIdentityEvidence,
) -> Result<String, IdentityDerivationDenial> {
    let composed = WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::CausalEvidenceReferenceReceipt,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("role"),
        "bridge-causal-evidence-reference",
    )
    .field_shape(WorthQueryEvidenceTag::new("owner"), owner)
    .field_shape(WorthQueryEvidenceTag::new("family"), family)
    .field_bridge_retained_evidence_identity(
        WorthQueryEvidenceTag::new("reference"),
        &bridge_reference,
    )
    .seal()?;
    Ok(composed.reporting_projection().to_string())
}
