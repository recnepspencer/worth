use serde::{Deserialize, Serialize};

use super::custom_rule::CustomInvariantProvenance;
use super::groups::{InvariantCostClass, InvariantGroupSet};
use super::results::{InvariantAdvisory, InvariantViolation};
use super::rule_id::CustomInvariantSemanticIdentity;
use super::rules::InvariantRule;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InvariantClass {
    AlwaysOnStructural,
    CommitBoundary,
    SnapshotAudit,
    HarnessHeavy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InvariantExecutionPoint {
    MutationSensitive,
    CommitBoundary,
    SnapshotPublication,
    CertificationBoundary,
    HarnessAudit,
    GraphComposition,
}

impl InvariantExecutionPoint {
    pub const COUNT: usize = 6;

    pub const fn class(self) -> InvariantClass {
        match self {
            Self::MutationSensitive => InvariantClass::AlwaysOnStructural,
            Self::CommitBoundary => InvariantClass::CommitBoundary,
            Self::SnapshotPublication => InvariantClass::SnapshotAudit,
            Self::CertificationBoundary => InvariantClass::SnapshotAudit,
            Self::HarnessAudit => InvariantClass::HarnessHeavy,
            Self::GraphComposition => InvariantClass::AlwaysOnStructural,
        }
    }

    pub const fn diagnostic_label(self) -> &'static str {
        match self {
            Self::MutationSensitive => "mutation_sensitive",
            Self::CommitBoundary => "commit_boundary",
            Self::SnapshotPublication => "snapshot_publication",
            Self::CertificationBoundary => "certification_boundary",
            Self::HarnessAudit => "harness_audit",
            Self::GraphComposition => "graph_composition",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InvariantFailureEffect {
    BlockCommit,
    BlockPublication,
    AuditOnly,
}

impl InvariantFailureEffect {
    pub const fn diagnostic_label(self) -> &'static str {
        match self {
            Self::BlockCommit => "block_commit",
            Self::BlockPublication => "block_publication",
            Self::AuditOnly => "audit_only",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InvariantVerdict {
    Pass,
    NotApplicable,
    Advisory {
        violation: InvariantViolation,
        advisory: InvariantAdvisory,
    },
    Violation(InvariantViolation),
}

impl InvariantVerdict {
    pub const fn diagnostic_label(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::NotApplicable => "not_applicable",
            Self::Advisory { .. } => "advisory",
            Self::Violation(_) => "violation",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InvariantReportedRule {
    Native(InvariantRule),
    Custom(CustomInvariantSemanticIdentity),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum InvariantWitnessBasis {
    #[default]
    StringOnly,
    Pass,
    UniqueEntityAspectField {
        field_locator: worth_foundational::facade::AspectFieldLocator,
        value: worth_foundational::facade::AspectValue,
        field_locator_canonical_bytes: Vec<u8>,
        value_canonical_bytes: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantWitnessKey {
    value: String,
    #[serde(skip)]
    basis: InvariantWitnessBasis,
}

impl InvariantWitnessKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            basis: InvariantWitnessBasis::StringOnly,
        }
    }

    pub fn pass() -> Self {
        Self {
            value: "pass".to_string(),
            basis: InvariantWitnessBasis::Pass,
        }
    }

    pub fn unique_entity_aspect_field(
        value: impl Into<String>,
        field_locator: worth_foundational::facade::AspectFieldLocator,
        aspect_value: worth_foundational::facade::AspectValue,
        field_locator_canonical_bytes: Vec<u8>,
        value_canonical_bytes: Vec<u8>,
    ) -> Self {
        Self {
            value: value.into(),
            basis: InvariantWitnessBasis::UniqueEntityAspectField {
                field_locator,
                value: aspect_value,
                field_locator_canonical_bytes,
                value_canonical_bytes,
            },
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn basis(&self) -> &InvariantWitnessBasis {
        &self.basis
    }
}

impl PartialEq for InvariantWitnessKey {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for InvariantWitnessKey {}

impl PartialOrd for InvariantWitnessKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for InvariantWitnessKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvariantDecisionKind {
    Passed,
    NotApplicable,
    Advisory,
    Violated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantDecisionRecord {
    pub execution_point: InvariantExecutionPoint,
    pub failure_effect: InvariantFailureEffect,
    pub rule: InvariantReportedRule,
    pub witness: InvariantWitnessKey,
    pub decision: InvariantDecisionKind,
    pub groups: InvariantGroupSet,
    pub cost: InvariantCostClass,
    pub custom_provenance_present: bool,
}

impl InvariantCheckResult {
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
            decision: match self.verdict {
                InvariantVerdict::Pass => InvariantDecisionKind::Passed,
                InvariantVerdict::NotApplicable => InvariantDecisionKind::NotApplicable,
                InvariantVerdict::Advisory { .. } => InvariantDecisionKind::Advisory,
                InvariantVerdict::Violation(_) => InvariantDecisionKind::Violated,
            },
            groups: self.groups,
            cost: self.cost,
            custom_provenance_present: self.custom_provenance.is_some(),
        }
    }
}
