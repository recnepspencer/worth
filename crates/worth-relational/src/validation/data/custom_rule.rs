mod errors;
mod refusal;
pub use refusal::CustomInvariantRefusal;
mod touched_scope;

pub use crate::validation::custom_rule::{
    CompleteStructuralWalk, CustomInvariantExecutionContext, CustomInvariantInterruption,
    CustomInvariantLeaseBudget, CustomInvariantNonApplicability, CustomInvariantProvenance,
    CustomInvariantReadBudget, CustomInvariantRegistration, CustomInvariantRegistrationError,
    CustomInvariantRule, CustomInvariantScopePlanner, CustomInvariantTraversalSummary,
    StructuralAspectStateView, StructuralReadError, StructuralRelationRecord,
    StructuralRelationView, StructuralWalkDenial, StructuralWalkDenialKind, StructuralWalkReads,
    StructuralWalkStep,
};
pub use errors::{
    CustomInvariantExecutionError, CustomInvariantPreparationError, CustomInvariantVerdict,
};
pub use touched_scope::{
    CustomInvariantTouchedSummary, PlannedRelationEndpointUpdate, StructuralCountView,
};
pub(crate) use touched_scope::{PlannedEntityCreate, PlannedRelationCreate, TouchedStructuralSet};

pub(crate) use crate::validation::custom_rule::PreparedCustomInvariantExecution;
pub(crate) use crate::validation::custom_rule::PreparedCustomInvariantScope;
pub(crate) use errors::{
    CustomInvariantFailure, CustomInvariantFailureKind, CustomInvariantRuntimePhase,
    CustomInvariantTraversalError, PreparedCustomInvariantExecutionOutcome,
};
