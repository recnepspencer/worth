mod artifact;
mod bridge;
mod decision;
mod outcome;
mod request;

pub(super) use artifact::{
    compose_causal_artifact_causal_identity, compose_causal_artifact_identity,
};
pub use bridge::bridge_causal_admission_summary_kind_label;
pub use bridge::{
    compose_bridge_causal_denial_identity, compose_bridge_causal_envelope_identity,
    compose_bridge_causal_envelope_receipt_identity,
    compose_bridge_causal_explanation_envelope_identity,
};
pub(super) use decision::{
    compose_causal_admission_counters_identity, compose_causal_admission_decision_identity,
    compose_causal_admission_receipt_identity, compose_causal_decision_trace_identity,
    compose_causal_decision_trace_row_identity,
};
pub(super) use outcome::{
    compose_causal_denied_artifact_detail_identity, compose_causal_materialized_detail_identity,
    compose_causal_outcome_identity,
};
pub(super) use request::{
    compose_causal_admission_subject_identity, compose_causal_inspection_request_failure_identity,
    compose_causal_inspection_request_identity, compose_causal_inspection_target_identity,
};

use crate::domain_computation::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::admission_decision::{
    CausalInspectionAdmissionDecision, CausalInspectionAdmissionDecisionKind,
    CausalInspectionAdmissionSubject,
};
use super::admission_trace::{
    CausalDecisionTraceRow, CausalInspectionAdmissionCounters, CausalInspectionAdmissionReceipt,
};
use super::inventory::CausalEvidenceFamily;
use super::materialization::{
    CausalBridgeReadmissionProof, CausalInspectionArtifactKind,
    CausalInspectionMaterializationPolicy, CausalInspectionRedactionPolicy,
    QueryCausalEvidenceReferenceArtifact,
};
use super::observation_identity::{
    CausalEvidenceReferenceDigest, CausalObservationAnchorDigest, CausalObservationTargetHandle,
    CausalResultShapeContextHandle,
};
use super::request::{CausalInspectionExplanationFamily, CausalInspectionRichness};
use worth_runtime_bridge::facade::{
    BridgeCausalEnvelopeDenial, BridgeCausalEnvelopeDenialKind, BridgeCausalEnvelopeIdentity,
    BridgeCausalEnvelopeReceipt, BridgeCausalEvidenceFamily, BridgeCausalExplanationEnvelope,
    BridgeCausalInspectionAdmissionSummaryKind,
};

macro_rules! causal_identity_type {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name(WorthQueryEvidenceIdentity);

        impl $name {
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }

            pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
                &self.0
            }
        }

        impl From<WorthQueryEvidenceIdentity> for $name {
            fn from(value: WorthQueryEvidenceIdentity) -> Self {
                Self(value)
            }
        }
    };
}

macro_rules! causal_identity_type_evidence_only {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name(WorthQueryEvidenceIdentity);

        impl $name {
            pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
                &self.0
            }
        }

        impl From<WorthQueryEvidenceIdentity> for $name {
            fn from(value: WorthQueryEvidenceIdentity) -> Self {
                Self(value)
            }
        }
    };
}

macro_rules! causal_identity_type_label_only {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name(WorthQueryEvidenceIdentity);

        impl $name {
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl From<WorthQueryEvidenceIdentity> for $name {
            fn from(value: WorthQueryEvidenceIdentity) -> Self {
                Self(value)
            }
        }
    };
}

causal_identity_type!(CausalInspectionTargetIdentity);
causal_identity_type!(CausalInspectionRequestIdentity);
causal_identity_type!(CausalInspectionRequestFailureIdentity);
causal_identity_type!(CausalInspectionAdmissionSubjectIdentity);
causal_identity_type!(CausalInspectionAdmissionDecisionIdentity);
causal_identity_type!(CausalInspectionDecisionTraceRowIdentity);
causal_identity_type!(CausalInspectionDecisionTraceIdentity);
causal_identity_type!(CausalInspectionAdmissionCountersIdentity);
causal_identity_type!(CausalInspectionAdmissionReceiptIdentity);
causal_identity_type!(CausalInspectionOutcomeIdentity);
causal_identity_type_evidence_only!(CausalInspectionMaterializedDetailIdentity);
causal_identity_type_evidence_only!(CausalInspectionDeniedArtifactDetailIdentity);
causal_identity_type!(CausalInspectionArtifactIdentity);
causal_identity_type!(CausalInspectionPerformanceSnapshotIdentity);
causal_identity_type!(CausalInspectionPerformanceSlopeIdentity);
causal_identity_type!(CausalInspectionPerformanceScaleSlopeIdentity);
causal_identity_type_label_only!(CausalInspectionPerformanceCertificationIdentity);
causal_identity_type_label_only!(CausalInspectionCertificationErrorIdentity);
causal_identity_type_label_only!(CausalInspectionCertificationFailureEvidenceIdentity);
