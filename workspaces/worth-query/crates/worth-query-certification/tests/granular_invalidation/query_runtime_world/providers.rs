use std::sync::Arc;

use worth_query::facade::domain as query_domain;

use super::super::contract::{TemporalDomain, TemporalDomainFamily, TemporalDomainOperation};

pub(super) struct EligibleProvider;

impl worth_runtime_bridge::facade::BridgeConditionalProviderSemantics for EligibleProvider {
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

impl worth_runtime_bridge::facade::BridgeConditionalWakeProvider for EligibleProvider {
    fn resolve(
        &self,
        _context: worth_runtime_bridge::facade::BridgeConditionalResolverContext,
    ) -> Result<worth_signal::facade::InstalledSignalConditionDecision, String> {
        Ok(worth_signal::facade::InstalledSignalConditionDecision::Eligible)
    }
}

pub(super) struct ConditionalCompute {
    pub(super) next_version: Arc<std::sync::atomic::AtomicU64>,
}

impl
    query_domain::WorthQueryConditionalNodeComputeProvider<
        TemporalDomain,
        TemporalDomainOperation,
        TemporalDomainFamily,
    > for ConditionalCompute
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

    fn execution_resource_support(&self) -> query_domain::WorthQueryExecutionResourceSupport {
        super::domain::resource_support()
    }

    fn compute(
        &self,
        _context: &query_domain::WorthQueryConditionalComputeContext,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        use std::sync::atomic::Ordering;
        let version = self.next_version.fetch_add(1, Ordering::SeqCst);
        Ok(worth_signal::facade::NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(
                worth_signal::facade::Aspect::new(0),
                version,
            )]),
        ))
    }
}
