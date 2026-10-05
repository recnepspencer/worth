//! What the delegation projection refuses, in closed Bank terms.

use worth_query_host::facade::primary_graph::{
    HandlerExecutionDenial, WorthQueryEntityResolutionDenial,
    WorthQueryInvariantDecisionPlanDenial, WorthQueryInvariantProjectionTraversalDenial,
};

#[derive(Debug)]
pub enum BankCapabilityDelegationProjectionDenial {
    EntityResolution(crate::BankEntityResolutionDenial),
    DecisionPlan(crate::BankInvariantDecisionPlanDenial),
    Traversal(crate::BankInvariantProjectionTraversalDenial),
    /// The requested child id already names a capability grant.
    ChildGrantExists,
    /// Query refused the child id selection outside resolution and its plan.
    ChildSelection(HandlerExecutionDenial),
}

impl std::fmt::Display for BankCapabilityDelegationProjectionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EntityResolution(denial) => denial.fmt(formatter),
            Self::DecisionPlan(denial) => denial.fmt(formatter),
            Self::Traversal(denial) => denial.fmt(formatter),
            Self::ChildGrantExists => {
                formatter.write_str("the delegated child id already names a capability grant")
            }
            Self::ChildSelection(denial) => denial.fmt(formatter),
        }
    }
}

impl std::error::Error for BankCapabilityDelegationProjectionDenial {}

/// The selection erases its denial: each type it reports keeps its Bank
/// description, and any other stays the denial Query reported.
pub(super) fn child_selection_denial(
    denial: HandlerExecutionDenial,
) -> BankCapabilityDelegationProjectionDenial {
    denial
        .downcast::<WorthQueryEntityResolutionDenial>()
        .map(Into::into)
        .or_else(|denial| {
            denial
                .downcast::<WorthQueryInvariantDecisionPlanDenial>()
                .map(Into::into)
        })
        .unwrap_or_else(BankCapabilityDelegationProjectionDenial::ChildSelection)
}

impl From<WorthQueryEntityResolutionDenial> for BankCapabilityDelegationProjectionDenial {
    fn from(value: WorthQueryEntityResolutionDenial) -> Self {
        Self::EntityResolution(crate::BankEntityResolutionDenial::from_query(value.kind()))
    }
}

impl From<WorthQueryInvariantDecisionPlanDenial> for BankCapabilityDelegationProjectionDenial {
    fn from(value: WorthQueryInvariantDecisionPlanDenial) -> Self {
        Self::DecisionPlan(crate::BankInvariantDecisionPlanDenial::from_query(
            value.kind(),
        ))
    }
}

impl From<WorthQueryInvariantProjectionTraversalDenial>
    for BankCapabilityDelegationProjectionDenial
{
    fn from(value: WorthQueryInvariantProjectionTraversalDenial) -> Self {
        Self::Traversal(crate::BankInvariantProjectionTraversalDenial::from_query(
            value.kind(),
        ))
    }
}
