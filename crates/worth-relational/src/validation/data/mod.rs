pub(crate) use worth_relational_contracts::internal_port::validation::data::catalog;
pub(crate) use worth_relational_contracts::internal_port::validation::data::contracts;
pub(crate) mod custom_rule;
pub(crate) use worth_relational_contracts::internal_port::validation::data::descriptor;
pub(crate) use worth_relational_contracts::internal_port::validation::data::execution;
pub(crate) use worth_relational_contracts::internal_port::validation::data::groups;
pub(crate) use worth_relational_contracts::internal_port::validation::data::results;
pub(crate) use worth_relational_contracts::internal_port::validation::data::rule_id;
pub(crate) use worth_relational_contracts::internal_port::validation::data::rules;
pub(crate) use worth_relational_contracts::internal_port::validation::data::structural_read;

pub use catalog::{InvariantCatalog, InvariantRegistration};
pub use contracts::InvariantPlanContract;
#[cfg(test)]
pub(crate) use custom_rule::TouchedStructuralSet;
pub use custom_rule::{
    CompleteStructuralWalk, CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantInterruption, CustomInvariantLeaseBudget, CustomInvariantNativeCheckpointDenial,
    CustomInvariantNonApplicability, CustomInvariantPreparationError, CustomInvariantReadBudget,
    CustomInvariantRefusal, CustomInvariantRegistration, CustomInvariantRegistrationError,
    CustomInvariantRule, CustomInvariantScopePlanner, CustomInvariantVerdict, InvariantEntityIds,
    InvariantPartitionIds, InvariantRelationIds, PlannedRelationEndpointUpdate,
    StructuralAspectStateView, StructuralReadError, StructuralRelationRecord,
    StructuralRelationView, StructuralWalkDenial, StructuralWalkDenialKind, StructuralWalkReads,
    StructuralWalkStep,
};
pub use descriptor::{
    CustomInvariantAccessContract, CustomInvariantDescriptor, CustomInvariantOperationalMetadata,
};
#[allow(unused_imports)]
pub use execution::InvariantWitnessBasis;
pub use execution::{
    InvariantClass, InvariantDecisionKind, InvariantDecisionRecord, InvariantExecutionPoint,
    InvariantFailureEffect, InvariantReportedRule, InvariantVerdict,
};
pub use groups::{InvariantCostClass, InvariantGroup, InvariantGroupSet};
pub use results::{
    CustomInvariantFailureIdentity, CustomInvariantFailureKind as ResultCustomInvariantFailureKind,
    CustomInvariantFailurePhase, InvariantViolation, InvariantViolationFields,
    RelationCardinalityBoundary,
};
pub use rule_id::{
    CustomInvariantRuleId, CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion,
};
pub use rules::{InvariantRule, RecordKindTag};

pub use custom_rule::{
    CustomInvariantAccessDenial, CustomInvariantAccessDenialKind, CustomInvariantFailureCause,
};

pub use crate::validation::engine::result::InvariantCheckResult;
pub(crate) use worth_relational_contracts::internal_port::validation::data::execution_posture;

#[cfg(test)]
pub(crate) use custom_rule::{
    CustomInvariantProvenance, CustomInvariantTraversalSummary, StructuralCountView,
};
