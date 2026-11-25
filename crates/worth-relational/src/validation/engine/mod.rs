pub(crate) use engine::{InvariantEngine, PendingProposedPass, StartedProposedPass};
pub(crate) use observation::InvariantObservation;
pub use observation::InvariantObservationKind;
#[cfg(test)]
pub(crate) use observation::InvariantSourceAffinity;
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarnessAuditMode {
    Disabled,
    Full,
}

#[cfg(test)]
impl HarnessAuditMode {
    pub(crate) const fn request_profile(self) -> Option<InvariantRequestProfile> {
        match self {
            Self::Disabled => None,
            Self::Full => Some(InvariantRequestProfile::HarnessAudit),
        }
    }
}
pub(crate) use profile::InvariantRequestProfile;
pub(crate) use request::{InvariantExecutionRequest, InvariantPreparationControl};
pub use result::{
    InvariantExecutionDisposition, InvariantExecutionMetadata, InvariantExecutionResult,
    InvariantFailure,
};
#[cfg(test)]
pub(crate) use result::{
    InvariantPlanScopeClass, InvariantProofBoundarySummary, InvariantScopeWideningCause,
};
pub(crate) use runtime_view::{InvariantRuntimeInputs, InvariantRuntimeView};
pub(crate) use worth_relational_mutation::internal_port::validation::engine::{
    engine, observation, profile, request, result, runtime_view, state_view,
};
#[cfg(test)]
mod tests;
