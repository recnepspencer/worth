//! Pure bridge signal decision classifier.
use worth_runtime_bridge::facade::BridgeConditionalDecisionEvidence;
use super::WorthQueryConditionalSignalDecision;

pub fn classify_bridge_signal(
    evidence: &BridgeConditionalDecisionEvidence,
) -> super::WorthQueryConditionalSignalDecision {
    super::execution_provenance::signal_decision(evidence.signal().class())
}
