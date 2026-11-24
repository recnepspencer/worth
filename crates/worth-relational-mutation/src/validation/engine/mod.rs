pub mod budget;
pub mod context;
pub mod engine;
pub mod evaluator;
pub mod input_preparation;
pub mod metrics;
pub mod observation;
pub mod policy;
pub mod profile;
pub mod request;
pub mod runtime_view;
pub mod stage_execution;
pub use runtime_view::{InvariantRuntimeInputs, InvariantRuntimeView};
pub mod result;
pub mod state_view;
#[cfg(test)]
pub mod test_view;
#[doc(hidden)]
pub mod tests;

pub(crate) use engine::{
    check_live as check_invariant_request, InvariantEngine, PendingProposedPass,
    StartedProposedPass,
};
pub use observation::InvariantObservationKind;
pub use observation::{InvariantObservation, InvariantSourceAffinity};
pub use profile::InvariantRequestProfile;
pub(crate) use request::{
    InvariantExecutionRequest, InvariantPreparationControl, PreparedRelationIntegrityScopes,
};
pub(crate) use result::InvariantExecutionSummary;
pub use result::{
    CustomInvariantTraceArtifact, InvariantExecutionDisposition, InvariantExecutionMetadata,
    InvariantExecutionResult, InvariantFailure, InvariantFailureArtifact, InvariantPlanScopeClass,
    InvariantProofBoundaryArtifact, InvariantProofBoundarySummary, InvariantScopeWideningCause,
};
