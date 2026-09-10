use super::*;
use crate::facade::{
    BridgeConditionalComparatorProvider, BridgeConditionalConditionProvider,
    BridgeConditionalProviderRole, BridgeConditionalResolverContext,
    BridgeConditionalTriggerProvider, BridgeConditionalWakeProvider,
};

use super::super::semantic_dependencies::{
    on_demand_contract, registered_comparator_contract, runtime_predicate_contract,
    temporal_contract,
};

struct Trigger(bool);

impl BridgeConditionalProviderSemantics for Trigger {
    type SemanticContract = bool;
    fn semantic_contract(&self) -> Self::SemanticContract {
        self.0
    }
    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        crate::facade::BridgeConditionalProviderHeapRetention,
        crate::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(crate::facade::BridgeConditionalProviderHeapRetention::none())
    }
}

impl BridgeConditionalTriggerProvider for Trigger {
    fn requested(&self) -> bool {
        self.0
    }
}

struct Decision(worth_signal::facade::InstalledSignalConditionDecision);

impl BridgeConditionalProviderSemantics for Decision {
    type SemanticContract = worth_signal::facade::InstalledSignalConditionDecision;
    fn semantic_contract(&self) -> Self::SemanticContract {
        self.0
    }
    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        crate::facade::BridgeConditionalProviderHeapRetention,
        crate::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(crate::facade::BridgeConditionalProviderHeapRetention::none())
    }
}

impl BridgeConditionalConditionProvider for Decision {
    fn resolve(
        &self,
        _context: BridgeConditionalResolverContext,
    ) -> Result<worth_signal::facade::InstalledSignalConditionDecision, String> {
        Ok(self.0)
    }
}

impl BridgeConditionalWakeProvider for Decision {
    fn resolve(
        &self,
        _context: BridgeConditionalResolverContext,
    ) -> Result<worth_signal::facade::InstalledSignalConditionDecision, String> {
        Ok(self.0)
    }
}

struct Comparator(bool);

impl BridgeConditionalProviderSemantics for Comparator {
    type SemanticContract = bool;
    fn semantic_contract(&self) -> Self::SemanticContract {
        self.0
    }
    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        crate::facade::BridgeConditionalProviderHeapRetention,
        crate::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(crate::facade::BridgeConditionalProviderHeapRetention::none())
    }
}

struct AlternateCompute(u64);

impl BridgeConditionalProviderSemantics for AlternateCompute {
    type SemanticContract = u64;
    fn semantic_contract(&self) -> Self::SemanticContract {
        self.0
    }
    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        crate::facade::BridgeConditionalProviderHeapRetention,
        crate::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(crate::facade::BridgeConditionalProviderHeapRetention::none())
    }
}

impl BridgeConditionalComputeProvider for AlternateCompute {
    fn compute(&self, _context: &mut dyn std::any::Any) -> Result<NodeEvaluationResult, String> {
        Ok(NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(
                worth_signal::facade::Aspect::new(0),
                self.0,
            )]),
        ))
    }
}

impl BridgeConditionalComparatorProvider for Comparator {
    fn has_meaningful_change(
        &self,
        _aspect: worth_signal::facade::Aspect,
        _cached: u64,
        _current: u64,
    ) -> Result<bool, String> {
        Ok(self.0)
    }
}

fn assert_provider_semantic_drift(
    contract: crate::facade::BridgeConditionalContract,
    current_providers: BridgeConditionalProviderSet,
    candidate_providers: BridgeConditionalProviderSet,
    role: BridgeConditionalProviderRole,
) {
    let (_current_owner, current) =
        install_with(contract.clone(), "bridge-main", current_providers);
    let (_candidate_owner, candidate) = install_with(contract, "bridge-main", candidate_providers);
    let denial = current.compare_semantic_continuity(&candidate).unwrap_err();
    assert!(matches!(
        denial.mismatch(),
        BridgeConditionalContinuityMismatch::ProviderSemanticContract { role: actual }
            if *actual == role
    ));
    assert!(denial.work().provider_roles_inspected() > 0);
}

#[test]
fn compute_configuration_drift_denies_continuity() {
    assert_provider_semantic_drift(
        conditional_contract("query:one"),
        BridgeConditionalProviderSet::new().compute(Compute(1)),
        BridgeConditionalProviderSet::new().compute(Compute(2)),
        BridgeConditionalProviderRole::Compute,
    );
}

#[test]
fn alternate_implementations_may_share_one_typed_semantic_contract() {
    let declaration = conditional_contract("query:one");
    let (_current_owner, current) = install_with(
        declaration.clone(),
        "bridge-main",
        BridgeConditionalProviderSet::new().compute(Compute(7)),
    );
    let (_candidate_owner, candidate) = install_with(
        declaration,
        "bridge-main",
        BridgeConditionalProviderSet::new().compute(AlternateCompute(7)),
    );

    let _continuity = current
        .compare_semantic_continuity(&candidate)
        .expect("typed owner contract, not provider implementation type, defines meaning");
}

#[test]
fn trigger_configuration_drift_denies_continuity() {
    assert_provider_semantic_drift(
        on_demand_contract("query:one"),
        BridgeConditionalProviderSet::new()
            .trigger(Trigger(true))
            .compute(Compute(1)),
        BridgeConditionalProviderSet::new()
            .trigger(Trigger(false))
            .compute(Compute(1)),
        BridgeConditionalProviderRole::Trigger,
    );
}

#[test]
fn wake_decision_drift_denies_continuity() {
    assert_provider_semantic_drift(
        temporal_contract("query:one"),
        BridgeConditionalProviderSet::new()
            .wake(Decision(
                worth_signal::facade::InstalledSignalConditionDecision::Eligible,
            ))
            .compute(Compute(1)),
        BridgeConditionalProviderSet::new()
            .wake(Decision(
                worth_signal::facade::InstalledSignalConditionDecision::Deferred,
            ))
            .compute(Compute(1)),
        BridgeConditionalProviderRole::Wake,
    );
}

#[test]
fn condition_decision_drift_denies_continuity() {
    assert_provider_semantic_drift(
        runtime_predicate_contract("query:one"),
        BridgeConditionalProviderSet::new()
            .condition(Decision(
                worth_signal::facade::InstalledSignalConditionDecision::Eligible,
            ))
            .compute(Compute(1)),
        BridgeConditionalProviderSet::new()
            .condition(Decision(
                worth_signal::facade::InstalledSignalConditionDecision::Deferred,
            ))
            .compute(Compute(1)),
        BridgeConditionalProviderRole::Condition,
    );
}

#[test]
fn comparator_configuration_drift_denies_continuity() {
    assert_provider_semantic_drift(
        registered_comparator_contract("query:one"),
        BridgeConditionalProviderSet::new()
            .dependency_comparator(Comparator(true))
            .compute(Compute(1)),
        BridgeConditionalProviderSet::new()
            .dependency_comparator(Comparator(false))
            .compute(Compute(1)),
        BridgeConditionalProviderRole::DependencyComparator,
    );
}
