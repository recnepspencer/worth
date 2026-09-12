mod catalog;
mod contracts;
mod custom_rule;
mod descriptor;
mod execution;
mod groups;
mod results;
mod rule_id;
mod rules;

pub(crate) use catalog::relation_integrity_registrations_for_plan;
pub use catalog::{InvariantCatalog, InvariantRegistration};
pub use contracts::InvariantPlanContract;
pub use custom_rule::{
    CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantPreparationError, CustomInvariantProvenance, CustomInvariantRegistration,
    CustomInvariantRegistrationError, CustomInvariantRule, CustomInvariantScopePlanner,
    CustomInvariantTouchedSummary, CustomInvariantTraversalSummary, CustomInvariantVerdict,
    PlannedRelationEndpointUpdate, StructuralCountView, StructuralRelationRecord,
    StructuralRelationView,
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
    InvariantRuleDescriptor, InvariantSemanticsClass, SupportedExecutionPoints,
};
#[allow(unused_imports)]
pub use execution::InvariantWitnessBasis;
pub use execution::{
    InvariantCheckResult, InvariantClass, InvariantDecisionKind, InvariantDecisionRecord,
    InvariantExecutionPoint, InvariantFailureEffect, InvariantReportedRule, InvariantVerdict,
    InvariantWitnessKey,
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
    InvariantRuleId, NativeInvariantRuleId,
};
pub use rules::{InvariantRule, RecordKindTag};
