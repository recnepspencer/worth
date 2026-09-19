use super::*;

pub(in crate::domain_computation::primary_graph) struct QueryConditionalComputeContext {
    pub(in crate::domain_computation::primary_graph) output_version: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct QueryConditionalComputeSemanticContract(pub(super) Arc<str>);

pub(super) struct QueryConditionalComputeProvider<Node> {
    pub(super) semantics: QueryConditionalComputeSemanticContract,
    pub(super) output_version:
        Option<Arc<dyn WorthQueryHostConditionalOutputVersionProvider<Node>>>,
}

impl<Node: 'static> BridgeConditionalProviderSemantics for QueryConditionalComputeProvider<Node> {
    type SemanticContract = QueryConditionalComputeSemanticContract;

    fn semantic_contract(&self) -> Self::SemanticContract {
        self.semantics.clone()
    }

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        let mut provider = vec![
            worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::arc_allocation_bytes(
                self.semantics.0.as_ref(),
            ),
        ];
        if let Some(output_version) = self.output_version.as_ref() {
            provider.push(
                worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::arc_allocation_bytes(
                    output_version.as_ref(),
                ),
            );
            provider.push(
                output_version
                    .retained_heap_bytes()
                    .map_err(|_| {
                        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow
                    })?
                    .bytes(),
            );
        }
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::try_from_parts(
            provider,
            [],
        )
    }
}

impl<Node: 'static> BridgeConditionalComputeProvider for QueryConditionalComputeProvider<Node> {
    fn compute(
        &self,
        context: &mut dyn std::any::Any,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        let context = context
            .downcast_ref::<QueryConditionalComputeContext>()
            .ok_or_else(|| {
                "conditional execution lacked Query's governed re-entry context".to_string()
            })?;
        let output_version = self
            .output_version
            .as_ref()
            .map(|provider| provider.output_version(context.output_version))
            .transpose()
            .map_err(|failure| failure.detail().to_owned())?
            .unwrap_or(context.output_version);
        Ok(worth_signal::facade::NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(
                worth_signal::facade::Aspect::new(0),
                output_version,
            )]),
        ))
    }
}

pub(super) struct QueryHostOutputComparator<Node> {
    pub(super) identity: Arc<str>,
    pub(super) provider: Arc<dyn WorthQueryHostConditionalOutputComparatorProvider<Node>>,
}

impl<Node: 'static> BridgeConditionalProviderSemantics for QueryHostOutputComparator<Node> {
    type SemanticContract = Arc<str>;

    fn semantic_contract(&self) -> Self::SemanticContract {
        Arc::clone(&self.identity)
    }

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::try_from_parts(
            [
                worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::arc_allocation_bytes(self.identity.as_ref()),
                worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::arc_allocation_bytes(self.provider.as_ref()),
                self.provider.retained_heap_bytes().map_err(|_| worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow)?.bytes(),
            ],
            [],
        )
    }
}

impl<Node: 'static> worth_runtime_bridge::facade::BridgeConditionalComparatorProvider
    for QueryHostOutputComparator<Node>
{
    fn has_meaningful_change(
        &self,
        _aspect: worth_signal::facade::Aspect,
        cached: u64,
        current: u64,
    ) -> Result<bool, String> {
        self.provider
            .has_meaningful_change(cached, current)
            .map_err(|failure| failure.detail().to_owned())
    }
}
