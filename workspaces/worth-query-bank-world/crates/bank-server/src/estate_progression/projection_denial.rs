//! Closed Bank descriptions of invariant-projection engine denials.

use worth_query_host::facade::primary_graph::{
    WorthQueryInvariantDecisionPlanDenialKind as QueryDecision,
    WorthQueryInvariantProjectionTraversalDenialKind as QueryTraversal,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankInvariantDecisionPlanDenial {
    UndeclaredDecisionTarget,
    ForeignIdentity,
    FieldNotInstalled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankInvariantProjectionTraversalDenial {
    RelationNotInstalled,
    UndeclaredDecisionTarget,
    ForeignIdentity,
    MutationTargetUnavailable,
    EndpointUnavailable,
    CardinalityContractMismatch,
    MissingTarget,
    MultipleTargets,
    WorkBudgetExceeded,
}

impl BankInvariantDecisionPlanDenial {
    pub const fn code(self) -> &'static str {
        match self {
            Self::UndeclaredDecisionTarget => "undeclared-decision-target",
            Self::ForeignIdentity => "foreign-identity",
            Self::FieldNotInstalled => "field-not-installed",
        }
    }

    pub(crate) const fn from_query(kind: QueryDecision) -> Self {
        match kind {
            QueryDecision::UndeclaredDecisionTarget => Self::UndeclaredDecisionTarget,
            QueryDecision::ForeignIdentity => Self::ForeignIdentity,
            QueryDecision::FieldNotInstalled => Self::FieldNotInstalled,
        }
    }
}

impl BankInvariantProjectionTraversalDenial {
    pub const fn code(self) -> &'static str {
        match self {
            Self::RelationNotInstalled => "relation-not-installed",
            Self::UndeclaredDecisionTarget => "undeclared-decision-target",
            Self::ForeignIdentity => "foreign-identity",
            Self::MutationTargetUnavailable => "mutation-target-unavailable",
            Self::EndpointUnavailable => "endpoint-unavailable",
            Self::CardinalityContractMismatch => "cardinality-contract-mismatch",
            Self::MissingTarget => "missing-target",
            Self::MultipleTargets => "multiple-targets",
            Self::WorkBudgetExceeded => "work-budget-exceeded",
        }
    }

    pub(crate) const fn from_query(kind: QueryTraversal) -> Self {
        match kind {
            QueryTraversal::RelationNotInstalled => Self::RelationNotInstalled,
            QueryTraversal::UndeclaredDecisionTarget => Self::UndeclaredDecisionTarget,
            QueryTraversal::ForeignIdentity => Self::ForeignIdentity,
            QueryTraversal::MutationTargetUnavailable => Self::MutationTargetUnavailable,
            QueryTraversal::EndpointUnavailable => Self::EndpointUnavailable,
            QueryTraversal::CardinalityContractMismatch => Self::CardinalityContractMismatch,
            QueryTraversal::MissingTarget => Self::MissingTarget,
            QueryTraversal::MultipleTargets => Self::MultipleTargets,
            QueryTraversal::WorkBudgetExceeded => Self::WorkBudgetExceeded,
        }
    }
}

impl std::fmt::Display for BankInvariantDecisionPlanDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::fmt::Display for BankInvariantProjectionTraversalDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}
