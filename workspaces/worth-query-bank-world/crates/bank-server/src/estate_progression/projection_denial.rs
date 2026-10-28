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
    ReadExecution(
        Result<
            worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind,
            worth_query_host::facade::primary_graph::WorthQueryProviderSessionControlStopKind,
        >,
    ),
    ExecutionRequest(
        worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial,
    ),
    DerivedSource(worth_query_host::facade::primary_graph::StructuralWalkDenialKind),
    ComponentSetNotInstalled,
    InvalidComponent,
    ComponentOutsideScope,
    RelationNotInstalled,
    NotOrbitRelation,
    InvalidOrbit,
    AggregateDefinitionUnavailable,
    AggregateIdentityUnavailable,
    InvalidAggregateLineage,
    UndeclaredDecisionTarget,
    ForeignIdentity,
    MutationTargetUnavailable,
    EndpointUnavailable,
    CardinalityContractMismatch,
    MissingTarget,
    MultipleTargets,
    MissingSource,
    MultipleSources,
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
            Self::ReadExecution(_) => "execution-request",
            Self::ExecutionRequest(_) => "execution-request",
            Self::DerivedSource(_) => "derived-source",
            Self::ComponentSetNotInstalled => "component-set-not-installed",
            Self::InvalidComponent => "invalid-component",
            Self::ComponentOutsideScope => "component-outside-scope",
            Self::RelationNotInstalled => "relation-not-installed",
            Self::NotOrbitRelation => "not-orbit-relation",
            Self::InvalidOrbit => "invalid-orbit",
            Self::AggregateDefinitionUnavailable => "aggregate-definition-unavailable",
            Self::AggregateIdentityUnavailable => "aggregate-identity-unavailable",
            Self::InvalidAggregateLineage => "invalid-aggregate-lineage",
            Self::UndeclaredDecisionTarget => "undeclared-decision-target",
            Self::ForeignIdentity => "foreign-identity",
            Self::MutationTargetUnavailable => "mutation-target-unavailable",
            Self::EndpointUnavailable => "endpoint-unavailable",
            Self::CardinalityContractMismatch => "cardinality-contract-mismatch",
            Self::MissingTarget => "missing-target",
            Self::MultipleTargets => "multiple-targets",
            Self::MissingSource => "missing-source",
            Self::MultipleSources => "multiple-sources",
            Self::WorkBudgetExceeded => "work-budget-exceeded",
        }
    }

    pub(crate) const fn from_query(kind: QueryTraversal) -> Self {
        match kind {
            QueryTraversal::ReadExecution(cause) => Self::ReadExecution(cause),
            QueryTraversal::ExecutionRequest(cause) => Self::ExecutionRequest(cause),
            QueryTraversal::DerivedSource(cause) => Self::DerivedSource(cause),
            QueryTraversal::ComponentSetNotInstalled => Self::ComponentSetNotInstalled,
            QueryTraversal::InvalidComponent => Self::InvalidComponent,
            QueryTraversal::ComponentOutsideScope => Self::ComponentOutsideScope,
            QueryTraversal::RelationNotInstalled => Self::RelationNotInstalled,
            QueryTraversal::NotOrbitRelation => Self::NotOrbitRelation,
            QueryTraversal::InvalidOrbit => Self::InvalidOrbit,
            QueryTraversal::AggregateDefinitionUnavailable => Self::AggregateDefinitionUnavailable,
            QueryTraversal::AggregateIdentityUnavailable => Self::AggregateIdentityUnavailable,
            QueryTraversal::InvalidAggregateLineage => Self::InvalidAggregateLineage,
            QueryTraversal::UndeclaredDecisionTarget => Self::UndeclaredDecisionTarget,
            QueryTraversal::ForeignIdentity => Self::ForeignIdentity,
            QueryTraversal::MutationTargetUnavailable => Self::MutationTargetUnavailable,
            QueryTraversal::EndpointUnavailable => Self::EndpointUnavailable,
            QueryTraversal::CardinalityContractMismatch => Self::CardinalityContractMismatch,
            QueryTraversal::MissingTarget => Self::MissingTarget,
            QueryTraversal::MultipleTargets => Self::MultipleTargets,
            QueryTraversal::MissingSource => Self::MissingSource,
            QueryTraversal::MultipleSources => Self::MultipleSources,
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
