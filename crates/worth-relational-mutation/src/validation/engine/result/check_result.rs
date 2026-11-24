//! An invariant decision, including the actual live proposal evidence when present.
use crate::contracts::validation::data::execution::{
    InvariantClass, InvariantDecisionKind, InvariantDecisionKindOwnerPort as _,
    InvariantDecisionRecord, InvariantExecutionPoint, InvariantFailureEffect,
    InvariantReportedRule, InvariantVerdict, InvariantWitnessKey,
};
use crate::contracts::validation::data::groups::{InvariantCostClass, InvariantGroupSet};
use crate::validation::custom_rule::CustomInvariantProvenance;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantCheckResult {
    pub execution_point: InvariantExecutionPoint,
    pub failure_effect: InvariantFailureEffect,
    pub rule: InvariantReportedRule,
    pub witness: InvariantWitnessKey,
    pub groups: InvariantGroupSet,
    pub cost: InvariantCostClass,
    pub custom_provenance: Option<CustomInvariantProvenance>,
    pub verdict: InvariantVerdict,
}

impl InvariantCheckResult {
    pub(crate) fn blocks_proposed_state(&self) -> bool {
        matches!(
            (self.execution_point, self.failure_effect),
            (
                InvariantExecutionPoint::CommitBoundary
                    | InvariantExecutionPoint::MutationSensitive,
                InvariantFailureEffect::BlockCommit
            ) | (
                InvariantExecutionPoint::SnapshotPublication,
                InvariantFailureEffect::BlockPublication
            )
        ) && matches!(self.verdict, InvariantVerdict::Violation(_))
    }

    pub fn class(&self) -> InvariantClass {
        self.execution_point.class()
    }

    pub fn groups(&self) -> InvariantGroupSet {
        self.groups
    }

    pub fn cost(&self) -> InvariantCostClass {
        self.cost
    }

    pub fn custom_provenance(&self) -> Option<&CustomInvariantProvenance> {
        self.custom_provenance.as_ref()
    }

    pub fn witness(&self) -> &InvariantWitnessKey {
        &self.witness
    }

    pub fn decision_record(&self) -> InvariantDecisionRecord {
        InvariantDecisionRecord {
            execution_point: self.execution_point,
            failure_effect: self.failure_effect,
            rule: self.rule.clone(),
            witness: self.witness.clone(),
            decision: InvariantDecisionKind::from_verdict(&self.verdict),
            groups: self.groups,
            cost: self.cost,
            custom_provenance_present: self.custom_provenance.is_some(),
        }
    }
}

impl<'a> From<&'a InvariantCheckResult>
    for crate::contracts::validation::engine::result::execution_summary::InvariantCheckObservation<
        'a,
    >
{
    fn from(check: &'a InvariantCheckResult) -> Self {
        Self {
            execution_point: check.execution_point,
            failure_effect: check.failure_effect,
            verdict: &check.verdict,
        }
    }
}
