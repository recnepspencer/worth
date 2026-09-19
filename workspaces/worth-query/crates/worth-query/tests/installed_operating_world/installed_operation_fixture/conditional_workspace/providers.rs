use worth_query::facade::domain;

use super::super::{GeometryDomain, ReadFamily, ReadVertex};

pub(super) fn providers_for(
    node: &domain::WorthQueryPortableConditionalNodeDeclaration,
) -> worth_runtime_bridge::facade::BridgeConditionalProviderSet {
    use domain::{
        WorthQueryArtifactReuseEquivalence as Reuse, WorthQueryComparatorRequirement as Dependency,
        WorthQueryConditionalConditionClass as Condition,
        WorthQueryOutputEquivalenceRequirement as Output,
    };
    let mut providers = worth_runtime_bridge::facade::BridgeConditionalProviderSet::new();
    if matches!(node.condition().class(), Condition::DomainSpecific) {
        providers = providers.condition(EligibleCondition);
    }
    if matches!(node.condition().class(), Condition::Temporal) {
        providers = providers.wake(EligibleCondition);
    }
    if matches!(node.condition().class(), Condition::OnDemand) {
        providers = providers.trigger(RequestedTrigger);
    }
    if matches!(node.dependency_comparator(), Dependency::Registered(_)) {
        providers = providers.dependency_comparator(ExactComparator);
    }
    if matches!(node.output_equivalence(), Output::Registered(_)) {
        providers = providers.output_comparator(ExactComparator);
    }
    if matches!(node.artifact_reuse_equivalence(), Reuse::Registered(_)) {
        providers = providers.reuse_comparator(ExactComparator);
    }
    providers
}

struct EligibleCondition;

impl worth_runtime_bridge::facade::BridgeConditionalProviderSemantics for EligibleCondition {
    type SemanticContract = ();

    fn semantic_contract(&self) -> Self::SemanticContract {}

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::none())
    }
}

impl worth_runtime_bridge::facade::BridgeConditionalConditionProvider for EligibleCondition {
    fn resolve(
        &self,
        _context: worth_runtime_bridge::facade::BridgeConditionalResolverContext,
    ) -> Result<worth_signal::facade::InstalledSignalConditionDecision, String> {
        Ok(worth_signal::facade::InstalledSignalConditionDecision::Eligible)
    }
}

impl worth_runtime_bridge::facade::BridgeConditionalWakeProvider for EligibleCondition {
    fn resolve(
        &self,
        _context: worth_runtime_bridge::facade::BridgeConditionalResolverContext,
    ) -> Result<worth_signal::facade::InstalledSignalConditionDecision, String> {
        Ok(worth_signal::facade::InstalledSignalConditionDecision::Eligible)
    }
}

struct RequestedTrigger;

impl worth_runtime_bridge::facade::BridgeConditionalProviderSemantics for RequestedTrigger {
    type SemanticContract = ();

    fn semantic_contract(&self) -> Self::SemanticContract {}

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::none())
    }
}

impl worth_runtime_bridge::facade::BridgeConditionalTriggerProvider for RequestedTrigger {
    fn requested(&self) -> bool {
        true
    }
}

struct ExactComparator;

impl worth_runtime_bridge::facade::BridgeConditionalProviderSemantics for ExactComparator {
    type SemanticContract = ();

    fn semantic_contract(&self) -> Self::SemanticContract {}

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::none())
    }
}

impl worth_runtime_bridge::facade::BridgeConditionalComparatorProvider for ExactComparator {
    fn has_meaningful_change(
        &self,
        _aspect: worth_signal::facade::Aspect,
        cached: u64,
        current: u64,
    ) -> Result<bool, String> {
        Ok(cached != current)
    }
}

pub(crate) struct DirectConditionalCompute;

impl domain::WorthQueryConditionalNodeComputeProvider<GeometryDomain, ReadVertex, ReadFamily>
    for DirectConditionalCompute
{
    type SemanticContract = ();

    fn semantic_contract(&self) -> Self::SemanticContract {}

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::none())
    }

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        crate::suite::installed_operation_fixture::execution_resource_support()
    }

    fn compute(
        &self,
        context: &domain::WorthQueryConditionalComputeContext,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        assert_eq!(context.execution_resources().strategy(), "fixture-bounded");
        assert_eq!(
            context
                .resource_envelope()
                .cancellation_safe_point()
                .as_str(),
            "fixture-chunk-boundary"
        );
        Ok(worth_signal::facade::NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(
                worth_signal::facade::Aspect::new(0),
                1,
            )]),
        ))
    }
}
