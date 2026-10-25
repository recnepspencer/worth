pub(crate) mod budget;
pub(crate) mod context;
mod engine;
pub(crate) mod evaluator;
pub(crate) mod input_preparation;
mod metrics;
mod observation;
mod policy;
mod profile;
mod request;
mod runtime_view;
mod stage_execution;
pub(crate) use runtime_view::InvariantRuntimeView;
mod result;
pub(crate) mod state_view;
#[cfg(test)]
mod tests;

pub(crate) use engine::{check_live as check_invariant_request, InvariantEngine};
pub(crate) use observation::InvariantObservation;
pub use observation::InvariantObservationKind;
#[cfg(test)]
pub use profile::HarnessAuditMode;
pub(crate) use profile::InvariantRequestProfile;
pub(crate) use request::{
    InvariantExecutionRequest, InvariantPreparationControl, PreparedRelationIntegrityScopes,
};
pub(crate) use result::InvariantExecutionSummary;
pub use result::{
    CustomInvariantTraceArtifact, InvariantExecutionDisposition, InvariantExecutionMetadata,
    InvariantExecutionResult, InvariantFailure, InvariantFailureArtifact, InvariantPlanScopeClass,
    InvariantProofBoundaryArtifact, InvariantProofBoundarySummary, InvariantScopeWideningCause,
};
