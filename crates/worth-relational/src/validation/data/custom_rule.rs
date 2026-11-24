pub(crate) mod access_denial;
pub(crate) mod failure_cause;
mod memory_denial;
pub use access_denial::{CustomInvariantAccessDenial, CustomInvariantAccessDenialKind};
pub use failure_cause::CustomInvariantFailureCause;
pub(crate) mod errors;
pub(crate) mod refusal;
pub use refusal::CustomInvariantRefusal;
pub(crate) mod touched_scope;

pub use crate::validation::custom_rule::{
    CompleteStructuralWalk, CustomInvariantExecutionContext, CustomInvariantInterruption,
    CustomInvariantLeaseBudget, CustomInvariantNativeCheckpointDenial,
    CustomInvariantNonApplicability, CustomInvariantProvenance, CustomInvariantReadBudget,
    CustomInvariantRegistration, CustomInvariantRegistrationError, CustomInvariantRule,
    CustomInvariantScopePlanner, CustomInvariantTraversalSummary, StructuralAspectStateView,
    StructuralReadError, StructuralRelationRecord, StructuralRelationView, StructuralWalkDenial,
    StructuralWalkDenialKind, StructuralWalkReads, StructuralWalkStep,
};
pub use errors::{
    CustomInvariantExecutionError, CustomInvariantPreparationError, CustomInvariantVerdict,
};
pub use touched_scope::{
    CustomInvariantTouchedSummary, InvariantEntityIds, InvariantPartitionIds, InvariantRelationIds,
    PlannedRelationEndpointUpdate, StructuralCountView,
};
pub(crate) use touched_scope::{PlannedEntityCreate, PlannedRelationCreate, TouchedStructuralSet};

pub(crate) use crate::validation::custom_rule::PreparedCustomInvariantExecution;
pub(crate) use crate::validation::custom_rule::PreparedCustomInvariantScope;
