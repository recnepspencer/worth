use std::collections::BTreeSet;

use worth_query_declaration::facade::domain_computation::WorthQueryResourceDimension;
use worth_query_declaration::facade::domain_computation::{
    WorthQueryRetainedProgressPosture, WorthQueryYieldedStatePosture,
};

use super::{WorthQueryExecutionResourceContract, WorthQueryInstalledBoundedStepContract};

pub(super) fn validate_resource_contract(
    contract: &WorthQueryExecutionResourceContract,
) -> Result<(), &'static str> {
    let WorthQueryExecutionResourceContract::Declared { strategies } = contract else {
        return Err("undeclared-execution-resource-contract");
    };
    if strategies.is_empty() {
        return Err("empty-execution-resource-strategy-set");
    }
    let mut names = BTreeSet::new();
    for strategy in strategies {
        if !names.insert(strategy.name().as_str()) {
            return Err("duplicate-execution-resource-strategy");
        }
        let envelope = strategy.envelope();
        if envelope.boundary() == worth_query_declaration::facade::domain_computation::WorthQueryExecutionBoundary::Atomic {
            super::atomic_validation::validate(envelope)?;
            continue;
        }
        if worth_query_declaration::facade::domain_computation::WorthQuerySemanticScaleAxis::ALL.into_iter()
            .any(|axis| axis != worth_query_declaration::facade::domain_computation::WorthQuerySemanticScaleAxis::WorkItems && envelope.optional_scale_ceiling(axis).is_none())
            || WorthQueryResourceDimension::ALL.into_iter().any(|dimension| envelope.optional_resource_ceiling(dimension).is_none()) {
            return Err("incomplete-bounded-step-envelope");
        }
        WorthQueryInstalledBoundedStepContract::derive(envelope)?;
        if strategy.envelope().yielded_state_posture()
            == WorthQueryYieldedStatePosture::ProviderCheckpoint
            && strategy.envelope().retained_progress_posture()
                != WorthQueryRetainedProgressPosture::RetainAttemptCapacity
        {
            return Err("provider-checkpoint-requires-retained-attempt-capacity");
        }
        if strategy
            .envelope()
            .resource_ceiling(WorthQueryResourceDimension::CleanupBudget)
            == 0
        {
            return Err("zero-cleanup-budget");
        }
    }
    Ok(())
}
