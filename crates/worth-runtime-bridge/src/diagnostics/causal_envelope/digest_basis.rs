#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BridgeCausalEnvelopeDigestArtifact {
    AdmissionSummary,
    AssemblyRequest,
    BindingSet,
    Counters,
    Denial,
    EvidenceBinding,
    EvidenceReference,
    ExplanationEnvelope,
    Identity,
    Receipt,
    NativeAdmissionSummary,
    NativeObservation,
    NativeSignalEvaluation,
    NativeSignalInvalidation,
    NativeEvidenceBinding,
    NativeCounters,
}

impl BridgeCausalEnvelopeDigestArtifact {
    pub(super) fn digest_domain(self) -> &'static str {
        match self {
            Self::AdmissionSummary => "bridge-causal-inspection-admission-summary",
            Self::AssemblyRequest => "bridge-causal-envelope-assembly-request",
            Self::BindingSet => "bridge-causal-envelope-bindings",
            Self::Counters => "bridge-causal-envelope-counters",
            Self::Denial => "bridge-causal-envelope-denial",
            Self::EvidenceBinding => "bridge-causal-evidence-binding",
            Self::EvidenceReference => "bridge-causal-evidence-reference",
            Self::ExplanationEnvelope => "bridge-causal-explanation-envelope",
            Self::Identity => "bridge-causal-envelope-identity",
            Self::Receipt => "bridge-causal-envelope-receipt",
            Self::NativeAdmissionSummary => "bridge-native-causal-admission-summary-v1",
            Self::NativeObservation => "bridge-native-conditional-observation-v1",
            Self::NativeSignalEvaluation => "bridge-native-signal-evaluation-v1",
            Self::NativeSignalInvalidation => "bridge-native-signal-invalidation-v1",
            Self::NativeEvidenceBinding => "bridge-native-causal-evidence-binding-v1",
            Self::NativeCounters => "bridge-native-causal-envelope-counters-v1",
        }
    }
}
