//! Authored output roles reconstruct host parameters through fresh installation.
use super::super::authored_provider_source as source;
use super::providers::computed_output;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_query_installation::facade::{
    WorthQueryAuthoredHostConditionalOutputComparatorProvider,
    WorthQueryAuthoredHostConditionalOutputVersionProvider,
};
use worth_runtime_bridge::facade::*;
use worth_signal::facade::{Aspect, NodeEvaluationResult};

pub(super) struct QueryAuthoredConditionalComputeProvider<Node> {
    node: Arc<str>,
    provider: Option<Arc<dyn WorthQueryAuthoredHostConditionalOutputVersionProvider<Node>>>,
    source: BridgeAuthoredProviderDescription,
}
impl<Node: 'static> QueryAuthoredConditionalComputeProvider<Node> {
    pub(super) fn new(
        node: Arc<str>,
        provider: Option<Arc<dyn WorthQueryAuthoredHostConditionalOutputVersionProvider<Node>>>,
    ) -> Self {
        let host = provider.as_ref().map(|p| p.authored_provider_description());
        let source = source::encode(
            "worth.query.conditional-output-version",
            &node,
            host.as_ref(),
        );
        Self {
            node,
            provider,
            source,
        }
    }
}
impl<Node: 'static> BridgeConditionalProviderSemantics
    for QueryAuthoredConditionalComputeProvider<Node>
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
        let mut parts =
            vec![BridgeConditionalProviderHeapRetention::arc_allocation_bytes(self.node.as_ref())];
        if let Some(provider) = &self.provider {
            parts.push(
                BridgeConditionalProviderHeapRetention::arc_allocation_bytes(provider.as_ref()),
            );
            parts.push(
                provider
                    .retained_heap_bytes()
                    .map_err(|_| BridgeConditionalProviderRetentionOverflow)?
                    .bytes(),
            );
        }
        BridgeConditionalProviderHeapRetention::try_from_parts(parts, [])
    }
}
impl<Node: 'static> BridgeConditionalComputeProvider
    for QueryAuthoredConditionalComputeProvider<Node>
{
    fn compute(&self, context: &mut dyn std::any::Any) -> Result<NodeEvaluationResult, String> {
        computed_output(context, None, |attempt| self.host_version(attempt))
    }

    fn compute_from_baseline(
        &self,
        context: &mut dyn std::any::Any,
        output_version_before: u64,
    ) -> Result<NodeEvaluationResult, String> {
        computed_output(context, Some(output_version_before), |attempt| {
            self.host_version(attempt)
        })
    }
}
impl<Node: 'static> QueryAuthoredConditionalComputeProvider<Node> {
    fn host_version(&self, attempt: u64) -> Option<Result<u64, String>> {
        self.provider.as_ref().map(|provider| {
            provider
                .output_version(attempt)
                .map_err(|cause| cause.detail().to_owned())
        })
    }
}
impl<Node: 'static> BridgeAuthoredConditionalProvider
    for QueryAuthoredConditionalComputeProvider<Node>
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
        let provider = match (&self.provider, decoded.host.as_ref()) {
            (None, None) => None,
            (Some(fresh), Some(source)) => Some(
                fresh
                    .readmit_authored_provider(source)
                    .map_err(BridgeAuthoredProviderDenial::Owner)?,
            ),
            (None, Some(_)) | (Some(_), None) => {
                return Err(BridgeAuthoredProviderDenial::InvalidPayload)
            }
        };
        Ok(Self::new(self.node.clone(), provider))
    }
}

pub(super) struct QueryAuthoredOutputComparator<Node> {
    node: Arc<str>,
    provider: Arc<dyn WorthQueryAuthoredHostConditionalOutputComparatorProvider<Node>>,
    source: BridgeAuthoredProviderDescription,
}
impl<Node: 'static> QueryAuthoredOutputComparator<Node> {
    pub(super) fn new(
        node: Arc<str>,
        provider: Arc<dyn WorthQueryAuthoredHostConditionalOutputComparatorProvider<Node>>,
    ) -> Self {
        let source = source::encode(
            "worth.query.conditional-output-comparator",
            &node,
            Some(&provider.authored_provider_description()),
        );
        Self {
            node,
            provider,
            source,
        }
    }
}
impl<Node: 'static> BridgeConditionalProviderSemantics for QueryAuthoredOutputComparator<Node> {
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
                BridgeConditionalProviderHeapRetention::arc_allocation_bytes(self.node.as_ref()),
                BridgeConditionalProviderHeapRetention::arc_allocation_bytes(
                    self.provider.as_ref(),
                ),
                self.provider
                    .retained_heap_bytes()
                    .map_err(|_| BridgeConditionalProviderRetentionOverflow)?
                    .bytes(),
            ],
            [],
        )
    }
}
impl<Node: 'static> BridgeConditionalComparatorProvider for QueryAuthoredOutputComparator<Node> {
    fn has_meaningful_change(&self, _: Aspect, cached: u64, current: u64) -> Result<bool, String> {
        self.provider
            .has_meaningful_change(cached, current)
            .map_err(|cause| cause.detail().to_owned())
    }
}
impl<Node: 'static> BridgeAuthoredConditionalProvider for QueryAuthoredOutputComparator<Node> {
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
        let provider = self
            .provider
            .readmit_authored_provider(child)
            .map_err(BridgeAuthoredProviderDenial::Owner)?;
        if provider.semantic_identity() != self.provider.semantic_identity() {
            return Err(BridgeAuthoredProviderDenial::InvalidPayload);
        }
        Ok(Self::new(self.node.clone(), provider))
    }
}
