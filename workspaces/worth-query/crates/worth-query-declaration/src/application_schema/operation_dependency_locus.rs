//! Owned declaration context, never a substitute for the rejected native guard.
use super::{
    ApplicationMutationPreconditionTarget, ApplicationOperationDecisionReadTarget,
    ApplicationOperationProgramTarget,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

/// The dependency being validated when an operation declaration is rejected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationOperationDependencyLocus {
    /// A declared operation program edge and its exact target.
    ProgramTarget(ApplicationOperationProgramTarget),
    /// A declared decision-read edge and its exact target.
    DecisionReadTarget(ApplicationOperationDecisionReadTarget),
    /// A mutation precondition requiring the corresponding declared read.
    MutationPrecondition(ApplicationMutationPreconditionTarget),
    /// Whole-body closure; no single field is asserted to be the cause.
    Body,
    /// A projection contract whose operation must be declared.
    ProjectionWorkBudget,
    /// An external effect's declared payload and operation closure.
    ExternalEffect {
        effect: String,
        payload_type: WorthQueryPortableTypeIdentity,
    },
    /// An inbound occurrence requiring its declared outbound effect.
    InboundOccurrence { effect: String },
    /// An aftermath contract whose operation must be declared.
    Aftermath,
}
