use worth_runtime_bridge::facade::{
    BridgeConditionalComputeProvider, BridgeConditionalProviderSemantics,
};

use super::installation::{
    WorthQueryConditionalComputeContext, WorthQueryConditionalNodeComputeProvider,
};

pub(crate) struct QueryComputeProvider<D, O, F, P> {
    provider: std::sync::Arc<P>,
    _marker: std::marker::PhantomData<fn() -> (D, O, F)>,
}

impl<D, O, F, P> QueryComputeProvider<D, O, F, P> {
    pub(crate) fn new(provider: std::sync::Arc<P>) -> Self {
        Self {
            provider,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<D: 'static, O: 'static, F: 'static, P> BridgeConditionalProviderSemantics
    for QueryComputeProvider<D, O, F, P>
where
    P: WorthQueryConditionalNodeComputeProvider<D, O, F>,
{
    type SemanticContract = P::SemanticContract;

    fn semantic_contract(&self) -> Self::SemanticContract {
        self.provider.semantic_contract()
    }

    fn retained_heap_bytes(
        &self,
        semantic_contract: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        let nested = self.provider.retained_heap_bytes(semantic_contract)?;
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::try_from_parts(
            [
                worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::arc_allocation_bytes(self.provider.as_ref()),
                nested.provider_state_bytes(),
            ],
            [nested.semantic_contract_state_bytes()],
        )
    }
}

impl<D: 'static, O: 'static, F: 'static, P> BridgeConditionalComputeProvider
    for QueryComputeProvider<D, O, F, P>
where
    P: WorthQueryConditionalNodeComputeProvider<D, O, F>,
{
    fn compute(
        &self,
        context: &mut dyn std::any::Any,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        let context = context
            .downcast_ref::<WorthQueryConditionalComputeContext>()
            .ok_or_else(|| {
                "conditional compute context belongs to another Query entry".to_string()
            })?;
        self.provider.compute(context)
    }
}
