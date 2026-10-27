use worth_query_execution::facade::causal_inspection::integration::{
    admission, admission_decision, admission_trace, anchor, builder, builder_support, inventory,
    materialization, observation_identity, receipt_types, reference, reference_index,
    reference_resolution, request,
};
// Operational inspection has one owner in the split execution engine.
mod certification;
mod domain_evidence_copy;
mod identity;
mod receipt;
mod receipt_helpers;

#[cfg(test)]
pub(crate) use admission::admit_causal_inspection;
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
pub use builder::{
    CausalInspection, CausalInspectionBasisMismatch, CausalInspectionEstimatedCost,
    CausalInspectionPlan, CausalInspectionPlanError, CausalInspectionPlanErrorKind,
    CausalInspectionPlanExplanation, CausalInspectionSupportPosture,
};
pub use builder_support::{
    CausalInspectionSupport, CausalInspectionSupportExplanation, CausalInspectionSupportRow,
    CausalInspectionSupportRowPosture,
};
pub use certification::{
    build_causal_inspection_certification_scope, certify_causal_inspection_runtime_path,
    CausalInspectionBoundaryAudit, CausalInspectionCertificationBundle,
    CausalInspectionCertificationError, CausalInspectionCertificationErrorKind,
    CausalInspectionCertificationFailureEvidence, CausalInspectionCertificationFailureKind,
    CausalInspectionCertificationFailureSource, CausalInspectionCertificationLane,
    CausalInspectionCertificationScope, CausalInspectionPerformanceCertificationBundle,
    CausalInspectionProofShapeCertification, CausalInspectionRepresentativeEvidence,
    CausalInspectionRepresentativeKind, CausalInspectionRepresentativeMatrix,
    CausalInspectionRepresentativeRowDigestSet, CausalInspectionScaleCounterSnapshot,
    CausalInspectionScaleFixtureSize, WorthQueryDomainEvidenceCertificationBundle,
    WorthQueryDomainEvidenceCertificationSidecar,
};
pub use domain_evidence_copy::{
    WorthQueryDomainEvidenceInspectionCopy, WorthQueryDomainEvidenceInspectionSidecar,
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
#[cfg(test)]
pub(crate) use observation_identity::{
    CausalObservationTargetHandle, CausalResultShapeContextHandle,
};
pub use receipt_types::{
    CausalInspectionReason, CausalObservationEvidenceIdentity, CausalObservationOutcome,
    QueryObservationReceiptFamily,
};
pub use reference::{
    CausalEvidenceReference, CausalEvidenceReferenceReceipt, CausalEvidenceReferenceResolution,
    CausalEvidenceReferenceResolutionCounters, CausalEvidenceReferenceResolutionDenial,
    CausalEvidenceReferenceSet,
};
#[cfg(test)]
pub(in crate::runtime) use reference_index::{
    causal_evidence_reference_index, causal_evidence_reference_index_record,
};
pub use reference_index::{
    CausalEvidenceReferenceIndex, CausalEvidenceReferenceIndexError,
    CausalEvidenceReferenceIndexErrorKind, CausalEvidenceReferenceIndexRecord,
};
pub use reference_resolution::resolve_causal_evidence_references;
#[cfg(test)]
pub(crate) use reference_resolution::resolve_indexed_causal_evidence_references;
pub(crate) use request::request_causal_inspection;
pub use request::{
    causal_inspection_target, CausalInspectionExplanationFamily, CausalInspectionRequest,
    CausalInspectionRequestError, CausalInspectionRequestErrorKind, CausalInspectionRichness,
    CausalInspectionTarget,
};

#[cfg(test)]
pub(crate) use identity::{
    causal_test_bridge_binding_reference_for_reporting,
    causal_test_compose_bridge_causal_denial_for_reporting,
    causal_test_compose_bridge_causal_envelope_identity_for_reporting,
    causal_test_compose_bridge_causal_envelope_receipt_identity_for_reporting,
    causal_test_compose_bridge_causal_explanation_envelope_identity_for_reporting,
};

pub use receipt::QueryObservationReceipt;
