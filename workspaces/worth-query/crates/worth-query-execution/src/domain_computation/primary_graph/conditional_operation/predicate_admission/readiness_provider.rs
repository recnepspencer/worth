use std::sync::Arc;

use worth_runtime_bridge::facade::{
    BridgeConditionalConditionProvider, BridgeConditionalProviderHeapRetention,
    BridgeConditionalProviderRetentionOverflow, BridgeConditionalProviderSemantics,
    BridgeConditionalResolverContext,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct QueryOutputReadinessSemanticContract {
    node_authority: Arc<str>,
    dependency_count: usize,
}

pub(super) struct QueryOutputReadinessPredicate {
    semantics: QueryOutputReadinessSemanticContract,
}

impl QueryOutputReadinessPredicate {
    pub(super) fn new(node_authority: Arc<str>, dependency_count: usize) -> Self {
        Self {
            semantics: QueryOutputReadinessSemanticContract {
                node_authority,
                dependency_count,
            },
        }
    }
}

impl BridgeConditionalProviderSemantics for QueryOutputReadinessPredicate {
    type SemanticContract = QueryOutputReadinessSemanticContract;

    fn semantic_contract(&self) -> Self::SemanticContract {
        self.semantics.clone()
    }

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<BridgeConditionalProviderHeapRetention, BridgeConditionalProviderRetentionOverflow>
    {
        Ok(BridgeConditionalProviderHeapRetention::new(
            BridgeConditionalProviderHeapRetention::arc_allocation_bytes(
                self.semantics.node_authority.as_ref(),
            ),
            0,
        ))
    }
}

impl BridgeConditionalConditionProvider for QueryOutputReadinessPredicate {
    fn resolve(
        &self,
        context: BridgeConditionalResolverContext,
    ) -> Result<worth_signal::facade::InstalledSignalConditionDecision, String> {
        let ready = context.observations().len() == self.semantics.dependency_count
            && context
                .observations()
                .iter()
                .all(|observation| observation.current().is_some());
        Ok(if ready {
            worth_signal::facade::InstalledSignalConditionDecision::Eligible
        } else {
            worth_signal::facade::InstalledSignalConditionDecision::Suppressed
        })
    }
}
