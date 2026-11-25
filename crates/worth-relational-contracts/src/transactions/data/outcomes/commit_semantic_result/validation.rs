//! Historical validation meaning excludes execution work and packet bookkeeping.
use crate::identity::data::VersionId;
use crate::validation::data::contracts::InvariantPlanContract;
use crate::validation::data::custom_rule::touched_scope::planned_records::{
    CustomInvariantTouchedSummary, StructuralCountView,
};
use crate::validation::data::execution::{
    InvariantDecisionKind, InvariantExecutionPoint, InvariantFailureEffect, InvariantReportedRule,
    InvariantVerdict,
};
use crate::validation::data::execution_posture::{
    InvariantExecutionDisposition, InvariantPlanScopeClass, InvariantScopeWideningCause,
};
use crate::validation::data::groups::{InvariantCostClass, InvariantGroupSet};
use crate::validation::data::non_applicability::CustomInvariantNonApplicability;
use crate::validation::data::observation_kind::InvariantObservationKind;
use serde::{Deserialize, Serialize};

pub mod production;
pub mod readmission;
pub mod readmission_memory;
pub use readmission_memory::{
    ValidationLeafMemory, ValidationReadmissionMemory, ValidationTouchedMemory,
};
pub mod summary;
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
/// Original validation facts; preparation posture and diagnostics stay in live execution metadata.
pub struct RelationalInvariantMetadataMeaning {
    pub execution_point: InvariantExecutionPoint,
    pub observation_kind: InvariantObservationKind,
    pub version: VersionId,
    pub current_version: VersionId,
    pub consumed_groups: InvariantGroupSet,
    pub applicable_groups: InvariantGroupSet,
    pub disposition: InvariantExecutionDisposition,
    pub plan_contract: Option<InvariantPlanContract>,
    pub has_merged_plan: bool,
    pub proof_scope: Option<RelationalInvariantProofScopeMeaning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalInvariantProofScopeMeaning {
    pub scope: InvariantPlanScopeClass,
    pub widened_causes: Vec<InvariantScopeWideningCause>,
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
