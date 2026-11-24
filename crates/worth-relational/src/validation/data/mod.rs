pub(crate) mod catalog;
pub(crate) mod compilation_denial;
pub(crate) mod contracts;
pub(crate) mod custom_rule;
pub(crate) mod descriptor;
pub(crate) mod execution;
pub(crate) mod groups;
pub(crate) mod observation_kind;
pub(crate) mod registration_conflict;
pub(crate) mod results;
pub(crate) mod rule_id;
pub(crate) mod rules;
pub(crate) mod structural_read;
pub(crate) mod structural_walk_denial;

pub(crate) use catalog::append_relation_integrity_registrations;
pub use catalog::{InvariantCatalog, InvariantRegistration};
pub use contracts::InvariantPlanContract;
pub use custom_rule::{
    CompleteStructuralWalk, CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantInterruption, CustomInvariantLeaseBudget, CustomInvariantNativeCheckpointDenial,
    CustomInvariantNonApplicability, CustomInvariantPreparationError, CustomInvariantProvenance,
    CustomInvariantReadBudget, CustomInvariantRefusal, CustomInvariantRegistration,
    CustomInvariantRegistrationError, CustomInvariantRule, CustomInvariantScopePlanner,
    CustomInvariantTouchedSummary, CustomInvariantTraversalSummary, CustomInvariantVerdict,
    InvariantEntityIds, InvariantPartitionIds, InvariantRelationIds, PlannedRelationEndpointUpdate,
    StructuralAspectStateView, StructuralCountView, StructuralReadError, StructuralRelationRecord,
    StructuralRelationView, StructuralWalkDenial, StructuralWalkDenialKind, StructuralWalkReads,
    StructuralWalkStep,
};
pub(crate) use custom_rule::{
    CustomInvariantFailure, CustomInvariantFailureKind, CustomInvariantRuntimePhase,
    PreparedCustomInvariantExecution, PreparedCustomInvariantExecutionOutcome,
    PreparedCustomInvariantScope,
};
pub(crate) use custom_rule::{
    CustomInvariantTraversalError, PlannedEntityCreate, PlannedRelationCreate, TouchedStructuralSet,
};
pub use descriptor::{
    CustomInvariantAccessContract, CustomInvariantDescriptor, CustomInvariantOperationalMetadata,
};
#[allow(unused_imports)]
pub use execution::InvariantWitnessBasis;
pub use execution::{
    InvariantClass, InvariantDecisionKind, InvariantDecisionRecord, InvariantExecutionPoint,
    InvariantFailureEffect, InvariantReportedRule, InvariantVerdict, InvariantWitnessKey,
};
pub use groups::{InvariantCostClass, InvariantGroup, InvariantGroupSet};
pub use results::{
    CustomInvariantFailureIdentity, CustomInvariantFailureKind as ResultCustomInvariantFailureKind,
    CustomInvariantFailurePhase, InvariantViolation, InvariantViolationFields,
    RelationCardinalityBoundary, RelationEndpointBoundary, StorageInconsistencyFailure,
    StorageInconsistencyLookup, StorageInconsistencyScan,
};
pub use rule_id::{
    CustomInvariantRuleId, CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion,
};
pub use rules::{InvariantRule, RecordKindTag};

pub use custom_rule::{
    CustomInvariantAccessDenial, CustomInvariantAccessDenialKind, CustomInvariantFailureCause,
};

pub(crate) mod execution_posture;
pub(crate) mod non_applicability;
pub use crate::validation::engine::result::InvariantCheckResult;
