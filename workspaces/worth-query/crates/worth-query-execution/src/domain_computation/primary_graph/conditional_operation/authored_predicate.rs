//! Authored predicates retain complete parameters and use the fresh host decoder.
use super::{
    authored_provider_source as source, predicate_observation::QueryTemporalPredicateProvider,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_query_installation::facade::{
    WorthQueryAuthoredHostPredicateBinding, WorthQueryHostConditionalPredicateProvider,
};
use worth_runtime_bridge::facade::*;

pub(in crate::domain_computation::primary_graph) struct QueryAuthoredPredicateProvider<
    Node,
    Provider,
> {
    binding: WorthQueryAuthoredHostPredicateBinding<Node, Provider>,
    evaluator: QueryTemporalPredicateProvider<Node, Provider>,
    node: Arc<str>,
    authority: Arc<str>,
    source: BridgeAuthoredProviderDescription,
}
impl<Node: 'static, Provider: WorthQueryHostConditionalPredicateProvider<Node>>
    QueryAuthoredPredicateProvider<Node, Provider>
{
    pub(in crate::domain_computation::primary_graph) fn new(
        binding: WorthQueryAuthoredHostPredicateBinding<Node, Provider>,
        node: Arc<str>,
        authority: Arc<str>,
    ) -> Self {
        let source = source::encode(
            "worth.query.conditional-predicate",
            &node,
            Some(&binding.description()),
        );
        let evaluator =
            QueryTemporalPredicateProvider::new(binding.provider().clone(), authority.clone());
        Self {
            binding,
            evaluator,
            node,
            authority,
            source,
        }
    }
}
impl<Node: 'static, Provider: WorthQueryHostConditionalPredicateProvider<Node>>
    BridgeConditionalProviderSemantics for QueryAuthoredPredicateProvider<Node, Provider>
{
    type SemanticContract = BridgeAuthoredProviderDescription;
    fn semantic_contract(&self) -> Self::SemanticContract {
        self.source.clone()
    }
    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<BridgeConditionalProviderHeapRetention, BridgeConditionalProviderRetentionOverflow>
    {
        BridgeConditionalProviderHeapRetention::try_from_parts(
            [
                self.binding
                    .retained_heap_bytes()
                    .map_err(|_| BridgeConditionalProviderRetentionOverflow)?
                    .bytes(),
                BridgeConditionalProviderHeapRetention::arc_allocation_bytes(self.node.as_ref()),
                BridgeConditionalProviderHeapRetention::arc_allocation_bytes(
                    self.authority.as_ref(),
                ),
            ],
            [],
        )
    }
}
impl<Node: 'static, Provider: WorthQueryHostConditionalPredicateProvider<Node>>
    BridgeConditionalWakeProvider for QueryAuthoredPredicateProvider<Node, Provider>
{
    fn resolve(
        &self,
        context: BridgeConditionalResolverContext,
    ) -> Result<worth_signal::facade::InstalledSignalConditionDecision, String> {
        self.evaluator.resolve(context)
    }
}
impl<Node: 'static, Provider: WorthQueryHostConditionalPredicateProvider<Node>>
    BridgeAuthoredConditionalProvider for QueryAuthoredPredicateProvider<Node, Provider>
{
    fn authored_description(&self) -> BridgeAuthoredProviderDescription {
        self.source.clone()
    }
    fn readmit_authored_provider(
        &self,
        description: &BridgeAuthoredProviderDescription,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, BridgeAuthoredProviderDenial> {
        self.admit_authored_source_version(description)?;
        let decoded = source::decode(description, execution)?;
        if decoded.node != self.node.as_ref() {
            return Err(BridgeAuthoredProviderDenial::InvalidPayload);
        }
        let child = decoded
            .host
            .as_ref()
            .ok_or(BridgeAuthoredProviderDenial::InvalidPayload)?;
        let binding = self
            .binding
            .readmit(child)
            .map_err(BridgeAuthoredProviderDenial::Owner)?;
        Ok(Self::new(
            binding,
            self.node.clone(),
            self.authority.clone(),
        ))
    }
}
