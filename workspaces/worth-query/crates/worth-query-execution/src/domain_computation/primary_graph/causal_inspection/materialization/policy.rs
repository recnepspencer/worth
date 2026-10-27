use crate::domain_computation::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceIdentityEncoder, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use worth_runtime_bridge::facade::BridgeCausalEnvelopeDenial;

use super::super::identity::compose_bridge_causal_denial_identity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalInspectionArtifactKind {
    Admitted,
    Advisory,
    Denied,
}

impl CausalInspectionArtifactKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Advisory => "advisory",
            Self::Denied => "denied",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalInspectionRedactionPolicy {
    PreserveDetail,
    DigestOnly,
}

impl CausalInspectionRedactionPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PreserveDetail => "preserve_detail",
            Self::DigestOnly => "digest_only",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalInspectionMaterializationPolicy {
    OfflineInterpretableArtifact,
    DigestReferenceOnly,
}

impl CausalInspectionMaterializationPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OfflineInterpretableArtifact => "offline_interpretable_artifact",
            Self::DigestReferenceOnly => "digest_reference_only",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalInspectionBoundaryEnvelopeCategory {
    PrimaryResult,
    StructuredWarnings,
    DecisionTrace,
    StructuralDeltas,
    IntegrityMarkers,
    PerformanceAccounting,
}

impl CausalInspectionBoundaryEnvelopeCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PrimaryResult => "primary_result",
            Self::StructuredWarnings => "structured_warnings",
            Self::DecisionTrace => "decision_trace",
            Self::StructuralDeltas => "structural_deltas",
            Self::IntegrityMarkers => "integrity_markers",
            Self::PerformanceAccounting => "performance_accounting",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalInspectionMaterializationErrorKind {
    AdmissionSummaryKindMismatch,
    AdmissionSummaryDigestMismatch,
    QueryObservationBindingMissing,
    QueryObservationBindingMismatch,
    QueryObservationBindingOverclaim,
    ReplayPostureUnsupported,
    MaterializationPolicyOverclaim,
}

impl CausalInspectionMaterializationErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AdmissionSummaryKindMismatch => "admission_summary_kind_mismatch",
            Self::AdmissionSummaryDigestMismatch => "admission_summary_digest_mismatch",
            Self::QueryObservationBindingMissing => "query_observation_binding_missing",
            Self::QueryObservationBindingMismatch => "query_observation_binding_mismatch",
            Self::QueryObservationBindingOverclaim => "query_observation_binding_overclaim",
            Self::ReplayPostureUnsupported => "replay_posture_unsupported",
            Self::MaterializationPolicyOverclaim => "materialization_policy_overclaim",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionMaterializationError {
    kind: CausalInspectionMaterializationErrorKind,
    failure_identity: WorthQueryEvidenceIdentity,
}

impl CausalInspectionMaterializationError {
    pub(super) fn new(
        kind: CausalInspectionMaterializationErrorKind,
        detail: impl FnOnce(WorthQueryEvidenceIdentityEncoder) -> WorthQueryEvidenceIdentityEncoder,
    ) -> Self {
        Self {
            kind,
            failure_identity: detail(
                WorthQueryEvidenceIdentity::compose(
                    WorthQueryEvidenceScope::CausalInspectionDeniedArtifactDetail,
                )
                .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str()),
            )
            .seal(),
        }
    }

    fn from_typed_bridge_denial(
        kind: CausalInspectionMaterializationErrorKind,
        denial: &BridgeCausalEnvelopeDenial,
    ) -> Self {
        let bridge_denial_identity = compose_bridge_causal_denial_identity(denial);
        Self {
            kind,
            failure_identity: WorthQueryEvidenceIdentity::compose(
                WorthQueryEvidenceScope::CausalInspectionDeniedArtifactDetail,
            )
            .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("bridge_denial"),
                &bridge_denial_identity,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("bridge_denial_kind"),
                denial.kind().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("bridge_denial_family"),
                denial.family().as_str(),
            )
            .seal(),
        }
    }

    pub fn kind(&self) -> CausalInspectionMaterializationErrorKind {
        self.kind
    }

    pub fn failure_digest(&self) -> &str {
        self.failure_identity.as_str()
    }

    pub(crate) fn from_bridge_assembly_denial(denial: &BridgeCausalEnvelopeDenial) -> Self {
        Self::from_typed_bridge_denial(
            CausalInspectionMaterializationErrorKind::MaterializationPolicyOverclaim,
            denial,
        )
    }
}

pub(super) fn boundary_categories() -> Vec<CausalInspectionBoundaryEnvelopeCategory> {
    vec![
        CausalInspectionBoundaryEnvelopeCategory::PrimaryResult,
        CausalInspectionBoundaryEnvelopeCategory::StructuredWarnings,
        CausalInspectionBoundaryEnvelopeCategory::DecisionTrace,
        CausalInspectionBoundaryEnvelopeCategory::StructuralDeltas,
        CausalInspectionBoundaryEnvelopeCategory::IntegrityMarkers,
        CausalInspectionBoundaryEnvelopeCategory::PerformanceAccounting,
    ]
}
