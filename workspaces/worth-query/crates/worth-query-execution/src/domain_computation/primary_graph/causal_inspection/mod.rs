mod admission;
mod admission_decision;
mod admission_trace;
mod anchor;
pub(crate) mod application_inspection;
mod application_observation;
pub(in crate::domain_computation) use application_inspection::WorthQueryApplicationInspectionTarget;
mod builder;
mod builder_bridge;
mod builder_support;
pub(crate) mod identity;
mod inventory;
pub(crate) mod materialization;
pub(crate) mod observation_identity;
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
pub use {application_inspection::DiagnosticComparisonDenial, application_inspection::FactlessCurrentness, crate::domain_computation::primary_graph::vocabulary::application_attempt::indexed_entity_selection::IndexedSelectionReobserveDenial, application_inspection::OutputFieldCaptureFailure, application_inspection::OwnEffectOnReads, crate::domain_computation::primary_graph::vocabulary::provider::diagnostic::RebaseVerificationReason, crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::StoreDenial, application_inspection::WorthQueryApplicationDisclosureContractDenial, application_inspection::WorthQueryApplicationInspectionCanonicalRefusal, application_inspection::WorthQueryApplicationInspectionDenial, application_inspection::WorthQueryApplicationInspectionOwnerRefusal, application_inspection::WorthQueryApplicationInspectionRequest, application_inspection::WorthQueryApplicationInspectionResourceRefusal, application_inspection::WorthQueryApplicationInspectionStage, application_inspection::WorthQueryApplicationInspectionStopKind, application_inspection::WorthQueryApplicationProducerConnection, application_inspection::WorthQueryApplicationProducerEvaluation, application_inspection::WorthQueryApplicationQueryGovernanceDenialKind, application_inspection::WorthQueryApplicationQueryInstallationDenial, application_inspection::WorthQueryApplicationQueryInstallationDenialKind, application_inspection::WorthQueryChangedInspectionRequest, application_inspection::WorthQueryChangedReferenceOnlyInspectionRequest, application_inspection::WorthQueryComparedApplicationInspection, application_inspection::WorthQueryConditionalHistoryAvailability, application_inspection::WorthQueryConditionalHistoryDecisionClass, application_inspection::WorthQueryConditionalHistoryOutcome, application_inspection::WorthQueryConnectedApplicationInspection, application_inspection::WorthQueryConnectedInspectionRequest, application_inspection::WorthQueryConnectedReferenceOnlyInspectionRequest, application_inspection::WorthQueryEvaluatedApplicationInspection, application_inspection::WorthQueryEvaluatedInspectionRequest, application_inspection::WorthQueryEvaluatedReferenceOnlyInspectionRequest, application_inspection::WorthQueryInspectionEvidenceAvailability, application_inspection::WorthQueryInspectionReadTerminalDenial, application_inspection::WorthQueryInspectionRetainedEvidence, application_inspection::WorthQueryInspectionSupplierEvidence, application_inspection::WorthQueryManagedGraphReadDenial, application_inspection::WorthQueryManagedGraphWorkSessionStartDenial, application_inspection::WorthQueryMaterializedConditionalHistory, application_inspection::WorthQueryMaterializedConnectedInspectionRequest, application_inspection::WorthQueryMaterializedPerformedSourceInspection, application_inspection::WorthQueryMaterializedSourceInspectionRequest, application_inspection::WorthQueryObservedApplicationInspection, application_inspection::WorthQueryObservedInspectionRequest, application_inspection::WorthQueryPerformedSourceInspection, application_inspection::WorthQueryPerformedSourceInspectionRequest, application_inspection::WorthQueryProducerConnectionDenial, application_inspection::WorthQueryProducerEvaluationDecision, application_inspection::WorthQueryProducerEvaluationDenial, application_inspection::WorthQueryProducerOutputConnectionDenial, application_inspection::WorthQueryReferenceOnlyInspectionRequest, application_inspection::WorthQuerySourceInspectionRequest, application_inspection::WorthQuerySourceReferenceOnlyInspectionRequest};
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

pub use super::application_contribution::WorthQueryProducerEvaluationIneligibility;

pub use application_inspection::RequestOps;
pub use application_inspection::SourceOps;
pub use application_inspection::{
    WorthQueryApplicationEvidenceFamily, WorthQueryApplicationMaterializationPolicy,
    WorthQueryApplicationMaterializationRequest, WorthQueryApplicationRichMaterialization,
    WorthQueryMaterializedApplicationInspection, WorthQueryMaterializedEvidenceReference,
    WorthQueryMaterializedFamilyOutcome, WorthQueryMaterializedFieldValue,
    WorthQueryMaterializedInspectionRequest, WorthQueryMaterializedQueryField,
    WorthQueryMaterializedQueryProjection, WorthQueryMaterializedQueryRelation,
    WorthQueryMaterializedQueryRow, WorthQueryMaterializedSourceEffects,
    WorthQueryMaterializedSourceField, WorthQueryMaterializedSourceFieldEffect,
    WorthQueryProjectionMaterializationRefusal,
};
pub(crate) mod host;
