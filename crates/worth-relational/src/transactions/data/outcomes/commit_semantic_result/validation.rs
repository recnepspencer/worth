//! Historical validation meaning excludes execution work and packet bookkeeping.
use crate::authority::commit::preparation::planning::strategy::PreparationStrategy;
use crate::identity::data::VersionId;
use crate::validation::data::{
    CustomInvariantNonApplicability, CustomInvariantTouchedSummary, InvariantCostClass,
    InvariantDecisionKind, InvariantExecutionPoint, InvariantFailureEffect, InvariantGroupSet,
    InvariantPlanContract, InvariantReportedRule, InvariantVerdict, StructuralCountView,
};
use crate::validation::engine::{
    InvariantExecutionDisposition, InvariantObservationKind, InvariantPlanScopeClass,
    InvariantScopeWideningCause,
};
use serde::{Deserialize, Serialize};

mod production;
mod readmission;
mod summary;
pub(super) use summary::summarize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalInvariantExecutionMeaning {
    pub metadata: RelationalInvariantMetadataMeaning,
    pub checks: Vec<RelationalInvariantCheckMeaning>,
    pub decisions: Vec<RelationalInvariantDecisionMeaning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalInvariantMetadataMeaning {
    pub execution_point: InvariantExecutionPoint,
    pub observation_kind: InvariantObservationKind,
    pub version: VersionId,
    pub current_version: VersionId,
    pub consumed_groups: InvariantGroupSet,
    pub applicable_groups: InvariantGroupSet,
    pub declared_cost: InvariantCostClass,
    pub disposition: InvariantExecutionDisposition,
    pub plan_contract: Option<InvariantPlanContract>,
    pub has_merged_plan: bool,
    pub preparation_strategy: Option<PreparationStrategy>,
    pub preparation_failures: Vec<RelationalValidationPreparationFailure>,
    pub proof_scope: Option<RelationalInvariantProofScopeMeaning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalInvariantProofScopeMeaning {
    pub scope: InvariantPlanScopeClass,
    pub widened_causes: Vec<InvariantScopeWideningCause>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalValidationPreparationFailure {
    PlanningProofInsufficient,
    ReductionIdentityConflict,
    SerialStrategySelected,
    PublicationIsolationViolation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalInvariantCustomProvenanceMeaning {
    pub observation_kind: InvariantObservationKind,
    pub version: VersionId,
    pub current_version: VersionId,
    pub touched: CustomInvariantTouchedSummary,
    pub counts: StructuralCountView,
    pub non_applicability: Option<CustomInvariantNonApplicability>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalInvariantCheckMeaning {
    pub execution_point: InvariantExecutionPoint,
    pub failure_effect: InvariantFailureEffect,
    pub rule: InvariantReportedRule,
    pub witness: String,
    pub groups: InvariantGroupSet,
    pub declared_cost: InvariantCostClass,
    pub custom_provenance: Option<RelationalInvariantCustomProvenanceMeaning>,
    pub verdict: InvariantVerdict,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalInvariantDecisionMeaning {
    pub execution_point: InvariantExecutionPoint,
    pub failure_effect: InvariantFailureEffect,
    pub rule: InvariantReportedRule,
    pub witness: String,
    pub decision: InvariantDecisionKind,
    pub groups: InvariantGroupSet,
    pub declared_cost: InvariantCostClass,
    pub custom_provenance_present: bool,
}
