//! Typed refusals of a stock recurring rule's schedule action and
//! installation; its typed lifecycle and gate are declaration vocabulary.
use worth_query_execution::facade::application_contribution::WorthQueryOutputReadinessContractDenial;
use worth_query_execution::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenial;

pub use worth_query_declaration::facade::application_schema::{
    WorthQueryRecurringGate, WorthQueryRecurringGateBinding, WorthQueryRecurringLifecycle,
    WorthQueryRecurringLifecycleBinding,
};

/// Why a recurring rule's schedule action refused before any effect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRecurringRuleRefusal {
    /// The rule identifier names the hub row that scopes every rule.
    HubRuleIdentifier,
    /// The rule identifier does not form a valid entity key.
    InvalidRuleKey,
}

impl std::fmt::Display for WorthQueryRecurringRuleRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::HubRuleIdentifier => "a recurring rule may not reuse the hub's identifier",
            Self::InvalidRuleKey => "the recurring rule identifier is not a valid entity key",
        })
    }
}

impl std::error::Error for WorthQueryRecurringRuleRefusal {}

/// A recurring rule installation refusal: `stage` names the part that
/// refused and `cause` its typed denial.
#[doc(hidden)]
pub fn recurring_installation_denial(
    stage: &str,
    cause: impl std::fmt::Debug,
) -> WorthQueryConditionalRuntimeInstallationDenial {
    WorthQueryOutputReadinessContractDenial::from_detail(format!(
        "recurring rule {stage} refused: {cause:?}"
    ))
    .into()
}
