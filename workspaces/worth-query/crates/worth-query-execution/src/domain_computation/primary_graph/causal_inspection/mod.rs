mod admission;
mod admission_decision;
mod admission_trace;
mod anchor;
mod application_observation;
mod builder;
mod builder_bridge;
mod builder_support;
mod identity;
mod inventory;
mod materialization;
mod observation_identity;
mod receipt_types;
mod reference;
mod reference_index;
mod reference_resolution;
mod request;

pub use admission::admit_causal_inspection;
pub use admission::{
    AdmittedCausalInspection, AdvisoryCausalInspection, CausalInspectionProofFlow,
    DeniedCausalInspection,
};
pub use admission_decision::{
    CausalInspectionAdmissionDecision, CausalInspectionAdmissionDecisionKind,
    CausalInspectionAdmissionSubject, CausalInspectionAdvisoryKind, CausalInspectionViolationKind,
};
pub use admission_trace::{
    CausalDecisionTraceIndex, CausalDecisionTraceRow, CausalInspectionAdmissionCounters,
    CausalInspectionAdmissionReceipt,
};
pub use anchor::{
    anchor_causal_observation, CausalObservationAnchor, CausalObservationAnchorCounters,
    CausalObservationAnchorError, CausalObservationAnchorErrorKind,
    CausalObservationMissingReferencePosture,
};
pub use application_observation::WorthQueryApplicationQueryObservation;
pub use builder::{
    CausalInspection, CausalInspectionBasisMismatch, CausalInspectionEstimatedCost,
    CausalInspectionPlan, CausalInspectionPlanError, CausalInspectionPlanErrorKind,
    CausalInspectionPlanExplanation, CausalInspectionSupportPosture,
};
pub use builder_support::{
    CausalInspectionSupport, CausalInspectionSupportExplanation, CausalInspectionSupportRow,
    CausalInspectionSupportRowPosture,
};
pub use inventory::{
    causal_evidence_inventory_rows, CausalEvidenceFamily, CausalEvidenceInventoryRow,
    CausalEvidenceOwner,
};
pub use materialization::{
    materialize_admitted_causal_inspection, materialize_advisory_causal_inspection,
    materialize_denied_causal_inspection, AdmittedQueryCausalInspectionArtifact,
    AdvisoryQueryCausalInspectionArtifact, CausalInspectionArtifactDecisionTrace,
    CausalInspectionArtifactIntegrity, CausalInspectionArtifactKind,
    CausalInspectionBoundaryEnvelopeCategory, CausalInspectionMaterializationError,
    CausalInspectionMaterializationErrorKind, CausalInspectionMaterializationPolicy,
    CausalInspectionPerformanceEnvelope, CausalInspectionRedactionPolicy,
    CausalMaterializationReceipt, DeniedQueryCausalInspectionArtifact,
    QueryCausalEvidenceReferenceArtifact, QueryCausalInspectionArtifact,
    QueryCausalTemporalAsyncExplanation, QueryCausalTemporalAsyncExplanationKind,
};
pub use observation_identity::{CausalEvidenceReferenceDigest, CausalObservationAnchorDigest};
pub use observation_identity::{CausalObservationTargetHandle, CausalResultShapeContextHandle};
pub use receipt_types::{
    CausalInspectionReason, CausalObservationEvidenceIdentity, CausalObservationOutcome,
    QueryObservationReceipt, QueryObservationReceiptFamily,
};
pub use reference::{
    CausalEvidenceReference, CausalEvidenceReferenceReceipt, CausalEvidenceReferenceResolution,
    CausalEvidenceReferenceResolutionCounters, CausalEvidenceReferenceResolutionDenial,
    CausalEvidenceReferenceSet,
};
pub use reference_index::{
    causal_evidence_reference_index, causal_evidence_reference_index_record,
};
pub use reference_index::{
    CausalEvidenceReferenceIndex, CausalEvidenceReferenceIndexError,
    CausalEvidenceReferenceIndexErrorKind, CausalEvidenceReferenceIndexRecord,
};
pub use reference_resolution::resolve_causal_evidence_references;
pub use reference_resolution::resolve_indexed_causal_evidence_references;
pub use request::request_causal_inspection;
pub use request::{
    causal_inspection_target, CausalInspectionExplanationFamily, CausalInspectionRequest,
    CausalInspectionRequestError, CausalInspectionRequestErrorKind, CausalInspectionRichness,
    CausalInspectionTarget,
};

/// Diagnostic integration for actual older-engine receipt and certification callers.
/// These descriptions cannot admit ordinary application inspection.
#[doc(hidden)]
pub mod integration {
    pub mod admission {
        pub use super::super::admission::*;
    }
    pub mod admission_decision {
        pub use super::super::admission_decision::*;
    }
    pub mod admission_trace {
        pub use super::super::admission_trace::*;
    }
    pub mod anchor {
        pub use super::super::anchor::*;
    }
    pub mod builder {
        pub use super::super::builder::*;
    }
    pub mod builder_support {
        pub use super::super::builder_support::*;
    }
    pub mod inventory {
        pub use super::super::inventory::*;
    }
    pub mod materialization {
        pub use super::super::materialization::*;
    }
    pub mod observation_identity {
        pub use super::super::observation_identity::*;
    }
    pub mod receipt_types {
        pub use super::super::receipt_types::*;
    }
    pub mod reference {
        pub use super::super::reference::*;
    }
    pub mod reference_index {
        pub use super::super::reference_index::*;
    }
    pub mod reference_resolution {
        pub use super::super::reference_resolution::*;
    }
    pub mod request {
        pub use super::super::request::*;
    }
    pub mod identity {
        pub use super::super::identity::{
            bridge_causal_admission_summary_kind_label, compose_bridge_causal_denial_identity,
            compose_bridge_causal_envelope_identity,
            compose_bridge_causal_envelope_receipt_identity,
            compose_bridge_causal_explanation_envelope_identity,
            CausalInspectionAdmissionCountersIdentity, CausalInspectionAdmissionDecisionIdentity,
            CausalInspectionAdmissionReceiptIdentity, CausalInspectionAdmissionSubjectIdentity,
            CausalInspectionArtifactIdentity, CausalInspectionCertificationErrorIdentity,
            CausalInspectionCertificationFailureEvidenceIdentity,
            CausalInspectionDecisionTraceIdentity, CausalInspectionDecisionTraceRowIdentity,
            CausalInspectionDeniedArtifactDetailIdentity,
            CausalInspectionMaterializedDetailIdentity, CausalInspectionOutcomeIdentity,
            CausalInspectionPerformanceCertificationIdentity,
            CausalInspectionPerformanceScaleSlopeIdentity,
            CausalInspectionPerformanceSlopeIdentity, CausalInspectionPerformanceSnapshotIdentity,
            CausalInspectionRequestFailureIdentity, CausalInspectionRequestIdentity,
            CausalInspectionTargetIdentity,
        };
    }
}
