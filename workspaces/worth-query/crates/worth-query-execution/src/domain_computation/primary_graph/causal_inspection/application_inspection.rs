//! Fresh application inspection owns admission, original target and its terminal.
mod admission;
mod connection;
mod source_original;
pub use connection::{
    WorthQueryApplicationProducerConnection, WorthQueryApplicationProducerEvaluation,
    WorthQueryProducerConnectionDenial, WorthQueryProducerEvaluationDecision,
    WorthQueryProducerEvaluationDenial, WorthQueryProducerOutputConnectionDenial,
};
mod identity;
mod report;
mod rich;
pub use rich::{
    WorthQueryApplicationEvidenceFamily, WorthQueryApplicationMaterializationPolicy,
    WorthQueryApplicationMaterializationRequest, WorthQueryApplicationRichMaterialization,
    WorthQueryMaterializedEvidenceReference, WorthQueryMaterializedFamilyOutcome,
    WorthQueryMaterializedFieldValue, WorthQueryMaterializedQueryField,
    WorthQueryMaterializedQueryProjection, WorthQueryMaterializedQueryRelation,
    WorthQueryMaterializedQueryRow, WorthQueryMaterializedSourceEffects,
    WorthQueryMaterializedSourceField, WorthQueryMaterializedSourceFieldEffect,
    WorthQueryProjectionMaterializationRefusal,
};
mod request;
mod result;
mod stop;
pub use request::{
    WorthQueryApplicationInspectionRequest, WorthQueryChangedInspectionRequest,
    WorthQueryChangedReferenceOnlyInspectionRequest, WorthQueryConnectedInspectionRequest,
    WorthQueryConnectedReferenceOnlyInspectionRequest, WorthQueryEvaluatedInspectionRequest,
    WorthQueryEvaluatedReferenceOnlyInspectionRequest,
    WorthQueryMaterializedConnectedInspectionRequest, WorthQueryMaterializedInspectionRequest,
    WorthQueryObservedInspectionRequest, WorthQueryPerformedSourceInspectionRequest,
    WorthQueryReferenceOnlyInspectionRequest, WorthQuerySourceInspectionRequest,
    WorthQuerySourceReferenceOnlyInspectionRequest,
};
pub use result::{
    WorthQueryComparedApplicationInspection, WorthQueryConnectedApplicationInspection,
    WorthQueryEvaluatedApplicationInspection, WorthQueryMaterializedApplicationInspection,
    WorthQueryObservedApplicationInspection, WorthQueryPerformedSourceInspection,
};
pub use stop::{
    WorthQueryApplicationInspectionCanonicalRefusal, WorthQueryApplicationInspectionDenial,
    WorthQueryApplicationInspectionResourceRefusal, WorthQueryApplicationInspectionStopKind,
};
#[cfg(test)]
#[path = "application_inspection/tests/scope_replacement.rs"]
mod scope_replacement;
mod target;
#[cfg(test)]
mod tests;
pub(in crate::domain_computation) use target::WorthQueryApplicationInspectionTarget;

#[cfg(test)]
#[path = "application_inspection/tests/comparison.rs"]
mod comparison_tests;

#[cfg(test)]
#[path = "application_inspection/tests/receipt_affinity.rs"]
mod receipt_affinity_tests;
